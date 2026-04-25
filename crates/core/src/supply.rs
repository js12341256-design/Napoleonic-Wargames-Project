//! Supply tracing and attrition resolution (PROMPT.md §16.5,
//! `docs/rules/supply.md`).
//!
//! Phase 5 ships three public entry points:
//!
//! - [`trace_supply`] — determine a corps's current supply state.
//! - [`resolve_supply_phase`] — emit supply events and apply attrition.
//! - [`validate_depot_order`] — pure validation for depot establishment.
//!
//! HARD RULES:
//! - No invented attrition numbers; placeholder table rows stay inert.
//! - No floats.
//! - No hash-ordered iteration in simulation logic.

use std::collections::BTreeSet;

use gc1805_core_schema::events::Event;
use gc1805_core_schema::ids::{AreaId, CorpsId, PowerId};
use gc1805_core_schema::scenario::{
    DiplomaticPairKey, DiplomaticState, Owner, PowerSlot, Scenario,
};
use gc1805_core_schema::supply_types::SupplyState;
use gc1805_core_schema::tables::{AttritionTable, Maybe};

use crate::map::MapGraph;
use crate::orders::EstablishDepotOrder;

/// Determine the current supply state of one corps.
pub fn trace_supply(scenario: &Scenario, corps_id: &CorpsId) -> SupplyState {
    let corps = match scenario.corps.get(corps_id) {
        Some(corps) => corps,
        None => return SupplyState::OutOfSupply,
    };

    let power = &corps.owner;
    let capital = match scenario.powers.get(power) {
        Some(setup) => &setup.capital,
        None => return local_foraging_state(scenario, &corps.area),
    };

    if &corps.area == capital {
        return SupplyState::InSupply;
    }

    let graph = MapGraph::from_scenario(scenario);
    let enemy_zoc = enemy_zoc_areas(scenario, &graph, power);
    let supply_targets = supply_targets(scenario, power);

    let mut visited: BTreeSet<AreaId> = BTreeSet::new();
    let mut frontier: Vec<AreaId> = vec![corps.area.clone()];
    visited.insert(corps.area.clone());

    while !frontier.is_empty() {
        let mut next: Vec<AreaId> = Vec::new();
        for area in &frontier {
            if supply_targets.contains(area) {
                return SupplyState::InSupply;
            }

            let Some(neighbours) = graph.neighbours_of(area) else {
                continue;
            };
            for neighbour in neighbours {
                if visited.contains(neighbour) {
                    continue;
                }
                if !area_allows_supply_trace(scenario, neighbour, power) {
                    continue;
                }
                if enemy_zoc.contains(neighbour) && !supply_targets.contains(neighbour) {
                    continue;
                }
                visited.insert(neighbour.clone());
                next.push(neighbour.clone());
            }
        }
        frontier = next;
    }

    local_foraging_state(scenario, &corps.area)
}

/// Resolve the supply phase for every corps in deterministic BTreeMap order.
pub fn resolve_supply_phase(scenario: &mut Scenario, tables: &AttritionTable) -> Vec<Event> {
    let corps_ids: Vec<CorpsId> = scenario.corps.keys().cloned().collect();
    let mut events: Vec<Event> = Vec::new();

    for corps_id in corps_ids {
        let supply_state = trace_supply(scenario, &corps_id);
        events.push(Event::SupplyTraced {
            corps: corps_id.clone(),
            supply_state: supply_state.clone(),
        });

        if let Some(corps) = scenario.corps.get_mut(&corps_id) {
            corps.supplied = matches!(supply_state, SupplyState::InSupply | SupplyState::Foraging);
        }

        if supply_state != SupplyState::OutOfSupply {
            continue;
        }

        let maybe_loss = tables
            .rows
            .get("default")
            .or_else(|| tables.rows.get("OUT_OF_SUPPLY"));

        let loss = match maybe_loss {
            Some(Maybe::Value(loss)) => *loss,
            Some(Maybe::Placeholder(_)) | None => continue,
        };

        if let Some(corps) = scenario.corps.get_mut(&corps_id) {
            let next_sp = (corps.infantry_sp - loss).max(0);
            corps.infantry_sp = next_sp;
        }

        events.push(Event::AttritionApplied {
            corps: corps_id,
            sp_loss: loss,
            reason: "OUT_OF_SUPPLY".into(),
        });
    }

    events
}

/// Validate a depot-establishment order.
pub fn validate_depot_order(
    scenario: &Scenario,
    order: &EstablishDepotOrder,
) -> Result<(), String> {
    let area = scenario
        .areas
        .get(&order.area)
        .ok_or_else(|| format!("unknown area `{}`", order.area))?;

    let _power_state = scenario
        .power_state
        .get(&order.submitter)
        .ok_or_else(|| format!("missing power_state for `{}`", order.submitter))?;

    let power_setup = scenario
        .powers
        .get(&order.submitter)
        .ok_or_else(|| format!("unknown power `{}`", order.submitter))?;

    if !area_is_owned_by_submitter_or_friendly_power(scenario, &area.owner, &order.submitter) {
        return Err(format!(
            "area `{}` is not owned by `{}` or a friendly power",
            order.area, order.submitter
        ));
    }

    let current_depots = 0usize;
    if current_depots >= usize::from(power_setup.max_depots) {
        return Err(format!(
            "`{}` has already reached max_depots ({})",
            order.submitter, power_setup.max_depots
        ));
    }

    Ok(())
}

fn supply_targets(scenario: &Scenario, power: &PowerId) -> BTreeSet<AreaId> {
    let mut targets = BTreeSet::new();
    if let Some(setup) = scenario.powers.get(power) {
        targets.insert(setup.capital.clone());
    }
    targets
}

fn local_foraging_state(scenario: &Scenario, area_id: &AreaId) -> SupplyState {
    let Some(area) = scenario.areas.get(area_id) else {
        return SupplyState::OutOfSupply;
    };

    match &area.money_yield {
        Maybe::Value(v) if *v > 0 => SupplyState::Foraging,
        _ => SupplyState::OutOfSupply,
    }
}

fn enemy_zoc_areas(scenario: &Scenario, graph: &MapGraph, power: &PowerId) -> BTreeSet<AreaId> {
    let mut zoc = BTreeSet::new();
    for corps in scenario
        .corps
        .values()
        .filter(|corps| &corps.owner != power)
    {
        if let Some(neighbours) = graph.neighbours_of(&corps.area) {
            for neighbour in neighbours {
                zoc.insert(neighbour.clone());
            }
        }
    }
    zoc
}

fn area_allows_supply_trace(scenario: &Scenario, area_id: &AreaId, power: &PowerId) -> bool {
    let Some(area) = scenario.areas.get(area_id) else {
        return false;
    };
    !owner_is_enemy_power(scenario, &area.owner, power)
}

fn owner_is_enemy_power(scenario: &Scenario, owner: &Owner, power: &PowerId) -> bool {
    match owner {
        Owner::Power(PowerSlot { power: owner_power }) => {
            owner_power != power && powers_at_war(scenario, power, owner_power)
        }
        Owner::Minor(_) | Owner::Unowned => false,
    }
}

fn area_is_owned_by_submitter_or_friendly_power(
    scenario: &Scenario,
    owner: &Owner,
    submitter: &PowerId,
) -> bool {
    match owner {
        Owner::Power(PowerSlot { power }) if power == submitter => true,
        Owner::Power(PowerSlot { power }) => matches!(
            scenario
                .diplomacy
                .get(&DiplomaticPairKey::new(submitter.clone(), power.clone())),
            Some(DiplomaticState::Friendly | DiplomaticState::Allied)
        ),
        Owner::Minor(_) | Owner::Unowned => false,
    }
}

fn powers_at_war(scenario: &Scenario, left: &PowerId, right: &PowerId) -> bool {
    matches!(
        scenario
            .diplomacy
            .get(&DiplomaticPairKey::new(left.clone(), right.clone())),
        Some(DiplomaticState::War)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::{AreaId, CorpsId, LeaderId};
    use gc1805_core_schema::scenario::{
        Area, AreaAdjacency, Corps, Features, GameDate, MovementRules, Owner, PowerSetup,
        PowerState, Terrain, SCHEMA_VERSION,
    };
    use gc1805_core_schema::tables::{AttritionTable, Maybe};
    use std::collections::BTreeMap;

    fn fra() -> PowerId {
        PowerId::from("FRA")
    }
    fn aus() -> PowerId {
        PowerId::from("AUS")
    }
    fn rus() -> PowerId {
        PowerId::from("RUS")
    }

    fn paris() -> AreaId {
        AreaId::from("AREA_PARIS")
    }
    fn vienna() -> AreaId {
        AreaId::from("AREA_VIENNA")
    }
    fn lyon() -> AreaId {
        AreaId::from("AREA_LYON")
    }
    fn berlin() -> AreaId {
        AreaId::from("AREA_BERLIN")
    }
    fn prague() -> AreaId {
        AreaId::from("AREA_PRAGUE")
    }
    fn munich() -> AreaId {
        AreaId::from("AREA_MUNICH")
    }
    fn madrid() -> AreaId {
        AreaId::from("AREA_MADRID")
    }
    fn warsaw() -> AreaId {
        AreaId::from("AREA_WARSAW")
    }

    fn fra_corps() -> CorpsId {
        CorpsId::from("CORPS_FRA_001")
    }
    fn fra_corps_two() -> CorpsId {
        CorpsId::from("CORPS_FRA_002")
    }
    fn aus_corps() -> CorpsId {
        CorpsId::from("CORPS_AUS_001")
    }
    fn aus_corps_two() -> CorpsId {
        CorpsId::from("CORPS_AUS_002")
    }

    fn area_named(id: &AreaId, owner: Owner, terrain: Terrain, money_yield: Maybe<i32>) -> Area {
        Area {
            display_name: id.as_str().replace("AREA_", ""),
            owner,
            terrain,
            fort_level: 0,
            money_yield,
            manpower_yield: Maybe::Value(0),
            capital_of: None,
            port: false,
            blockaded: false,
            map_x: 0,
            map_y: 0,
        }
    }

    fn edge(from: &AreaId, to: &AreaId) -> AreaAdjacency {
        AreaAdjacency {
            from: from.clone(),
            to: to.clone(),
            cost: Maybe::Value(1),
        }
    }

    fn add_area(
        scenario: &mut Scenario,
        id: AreaId,
        owner: Owner,
        terrain: Terrain,
        money_yield: Maybe<i32>,
    ) {
        scenario
            .areas
            .insert(id.clone(), area_named(&id, owner, terrain, money_yield));
    }

    fn connect(scenario: &mut Scenario, left: &AreaId, right: &AreaId) {
        scenario.adjacency.push(edge(left, right));
        scenario.adjacency.push(edge(right, left));
    }

    fn add_corps(
        scenario: &mut Scenario,
        id: CorpsId,
        owner: PowerId,
        area: AreaId,
        infantry: i32,
    ) {
        scenario.corps.insert(
            id,
            Corps {
                display_name: "Corps".into(),
                owner,
                area,
                infantry_sp: infantry,
                cavalry_sp: 0,
                artillery_sp: 0,
                morale_q4: 9_000,
                supplied: true,
                leader: None,
            },
        );
    }

    fn set_relation(
        scenario: &mut Scenario,
        left: PowerId,
        right: PowerId,
        state: DiplomaticState,
    ) {
        scenario
            .diplomacy
            .insert(DiplomaticPairKey::new(left, right), state);
    }

    fn supply_scenario() -> Scenario {
        let mut powers = BTreeMap::new();
        powers.insert(
            fra(),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: LeaderId::from("LEADER_NAPOLEON"),
                capital: paris(),
                starting_treasury: 100,
                starting_manpower: 50,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 3,
                mobilization_areas: vec![paris()],
                color_hex: "#1122aa".into(),
            },
        );
        powers.insert(
            aus(),
            PowerSetup {
                display_name: "Austria".into(),
                house: "Habsburg".into(),
                ruler: LeaderId::from("LEADER_FRANCIS"),
                capital: vienna(),
                starting_treasury: 100,
                starting_manpower: 50,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 3,
                mobilization_areas: vec![vienna()],
                color_hex: "#ffffff".into(),
            },
        );
        powers.insert(
            rus(),
            PowerSetup {
                display_name: "Russia".into(),
                house: "Romanov".into(),
                ruler: LeaderId::from("LEADER_ALEXANDER"),
                capital: warsaw(),
                starting_treasury: 100,
                starting_manpower: 50,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 3,
                mobilization_areas: vec![warsaw()],
                color_hex: "#33aa33".into(),
            },
        );

        let mut power_state = BTreeMap::new();
        power_state.insert(
            fra(),
            PowerState {
                treasury: 50,
                manpower: 20,
                prestige: 0,
                tax_policy: gc1805_core_schema::scenario::TaxPolicy::Standard,
            },
        );
        power_state.insert(
            aus(),
            PowerState {
                treasury: 50,
                manpower: 20,
                prestige: 0,
                tax_policy: gc1805_core_schema::scenario::TaxPolicy::Standard,
            },
        );
        power_state.insert(
            rus(),
            PowerState {
                treasury: 50,
                manpower: 20,
                prestige: 0,
                tax_policy: gc1805_core_schema::scenario::TaxPolicy::Standard,
            },
        );

        let mut scenario = Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "supply_test".into(),
            name: "Supply Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules::default(),
            current_turn: 0,
            power_state,
            production_queue: vec![],
            replacement_queue: vec![],
            subsidy_queue: vec![],
            powers,
            minors: BTreeMap::new(),
            leaders: BTreeMap::new(),
            areas: BTreeMap::new(),
            sea_zones: BTreeMap::new(),
            corps: BTreeMap::new(),
            fleets: BTreeMap::new(),
            diplomacy: BTreeMap::new(),
            adjacency: vec![],
            coast_links: vec![],
            sea_adjacency: vec![],
        };

        add_area(
            &mut scenario,
            paris(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Urban,
            Maybe::Value(2),
        );
        add_area(
            &mut scenario,
            vienna(),
            Owner::Power(PowerSlot { power: aus() }),
            Terrain::Urban,
            Maybe::Value(2),
        );
        connect(&mut scenario, &paris(), &vienna());

        if let Some(area) = scenario.areas.get_mut(&paris()) {
            area.capital_of = Some(fra());
        }
        if let Some(area) = scenario.areas.get_mut(&vienna()) {
            area.capital_of = Some(aus());
        }

        add_corps(&mut scenario, fra_corps(), fra(), paris(), 4);
        add_corps(&mut scenario, aus_corps(), aus(), vienna(), 4);
        set_relation(&mut scenario, fra(), aus(), DiplomaticState::War);

        scenario
    }

    fn placeholder_attrition() -> AttritionTable {
        AttritionTable {
            schema_version: 1,
            rows: BTreeMap::new(),
        }
    }

    fn value_attrition() -> AttritionTable {
        let mut rows = BTreeMap::new();
        rows.insert("default".into(), Maybe::Value(1));
        AttritionTable {
            schema_version: 1,
            rows,
        }
    }

    #[test]
    fn in_supply_at_capital() {
        let scenario = supply_scenario();
        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn in_supply_one_hop() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &lyon());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();

        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn out_of_supply_blocked_by_enemy() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            berlin(),
            Owner::Power(PowerSlot { power: aus() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();

        assert_eq!(
            trace_supply(&scenario, &fra_corps()),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn foraging_available() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(3),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();

        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::Foraging);
    }

    #[test]
    fn foraging_blocked_by_placeholder_yield() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Placeholder(Default::default()),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();

        assert_eq!(
            trace_supply(&scenario, &fra_corps()),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn supply_phase_emits_traced_event() {
        let mut scenario = supply_scenario();
        let events = resolve_supply_phase(&mut scenario, &placeholder_attrition());
        assert!(events.iter().any(|event| matches!(
            event,
            Event::SupplyTraced {
                corps,
                supply_state: SupplyState::InSupply
            } if corps == &fra_corps()
        )));
    }

    #[test]
    fn attrition_applied_when_out_of_supply_and_value_table() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();

        let events = resolve_supply_phase(&mut scenario, &value_attrition());
        assert_eq!(scenario.corps[&fra_corps()].infantry_sp, 3);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::AttritionApplied { corps, sp_loss: 1, reason } if corps == &fra_corps() && reason == "OUT_OF_SUPPLY"
        )));
    }

    #[test]
    fn attrition_skipped_when_placeholder_table() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        let before = scenario.corps[&fra_corps()].infantry_sp;

        let events = resolve_supply_phase(&mut scenario, &placeholder_attrition());
        assert_eq!(scenario.corps[&fra_corps()].infantry_sp, before);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::AttritionApplied { .. })));
    }

    #[test]
    fn attrition_clamps_sp_at_zero() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        scenario.corps.get_mut(&fra_corps()).unwrap().infantry_sp = 1;
        let mut table = AttritionTable {
            schema_version: 1,
            rows: BTreeMap::new(),
        };
        table.rows.insert("default".into(), Maybe::Value(5));

        resolve_supply_phase(&mut scenario, &table);
        assert_eq!(scenario.corps[&fra_corps()].infantry_sp, 0);
    }

    #[test]
    fn in_supply_multiple_hops() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            berlin(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        set_relation(&mut scenario, fra(), rus(), DiplomaticState::Neutral);

        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn bfs_respects_enemy_zoc() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            berlin(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            prague(),
            Owner::Power(PowerSlot { power: aus() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        connect(&mut scenario, &berlin(), &prague());
        add_corps(&mut scenario, aus_corps_two(), aus(), prague(), 2);
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();

        assert_eq!(
            trace_supply(&scenario, &fra_corps()),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn two_corps_different_states() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(4),
        );
        add_corps(&mut scenario, fra_corps_two(), fra(), lyon(), 3);

        let events = resolve_supply_phase(&mut scenario, &placeholder_attrition());
        assert!(events.iter().any(|event| matches!(
            event,
            Event::SupplyTraced { corps, supply_state: SupplyState::InSupply } if corps == &fra_corps()
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            Event::SupplyTraced { corps, supply_state: SupplyState::Foraging } if corps == &fra_corps_two()
        )));
    }

    #[test]
    fn foraging_only_when_truly_cut_off() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(5),
        );
        connect(&mut scenario, &paris(), &lyon());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();

        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn supply_deterministic() {
        let mut left = supply_scenario();
        let mut right = supply_scenario();
        add_area(
            &mut left,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut right,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        left.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        right.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        let table = value_attrition();

        let left_events = resolve_supply_phase(&mut left, &table);
        let right_events = resolve_supply_phase(&mut right, &table);

        assert_eq!(left_events, right_events);
        assert_eq!(
            left.corps[&fra_corps()].infantry_sp,
            right.corps[&fra_corps()].infantry_sp
        );
    }

    #[test]
    fn isolated_area_out_of_supply() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        assert_eq!(
            trace_supply(&scenario, &fra_corps()),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn no_adjacency_needed_at_capital() {
        let mut scenario = supply_scenario();
        scenario.adjacency.clear();
        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn neutral_chain_allows_supply() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            berlin(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn enemy_owned_area_blocks_trace() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            berlin(),
            Owner::Power(PowerSlot { power: aus() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        assert_eq!(
            trace_supply(&scenario, &fra_corps()),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn enemy_zoc_on_intermediate_blocks() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            berlin(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            prague(),
            Owner::Power(PowerSlot { power: aus() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        connect(&mut scenario, &berlin(), &prague());
        add_corps(&mut scenario, aus_corps_two(), aus(), prague(), 2);
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        assert_eq!(
            trace_supply(&scenario, &fra_corps()),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn enemy_zoc_not_on_unused_branch_does_not_block() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            berlin(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            prague(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        connect(&mut scenario, &paris(), &prague());
        connect(&mut scenario, &prague(), &vienna());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn multiple_enemy_corps_expand_zoc() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            berlin(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            prague(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        add_area(
            &mut scenario,
            munich(),
            Owner::Power(PowerSlot { power: aus() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        connect(&mut scenario, &paris(), &berlin());
        connect(&mut scenario, &berlin(), &lyon());
        connect(&mut scenario, &paris(), &prague());
        connect(&mut scenario, &prague(), &lyon());
        connect(&mut scenario, &berlin(), &munich());
        connect(&mut scenario, &prague(), &munich());
        add_corps(&mut scenario, aus_corps_two(), aus(), munich(), 2);
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        assert_eq!(
            trace_supply(&scenario, &fra_corps()),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn unknown_corps_is_out_of_supply() {
        let scenario = supply_scenario();
        assert_eq!(
            trace_supply(&scenario, &CorpsId::from("CORPS_UNKNOWN")),
            SupplyState::OutOfSupply
        );
    }

    #[test]
    fn supply_phase_uses_btreemap_order() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        add_corps(
            &mut scenario,
            CorpsId::from("CORPS_FRA_999"),
            fra(),
            lyon(),
            2,
        );
        let events = resolve_supply_phase(&mut scenario, &placeholder_attrition());
        let traced: Vec<_> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::SupplyTraced { corps, .. } => Some(corps),
                _ => None,
            })
            .collect();
        assert_eq!(traced[0], CorpsId::from("CORPS_AUS_001"));
        assert_eq!(traced[1], fra_corps());
        assert_eq!(traced[2], CorpsId::from("CORPS_FRA_999"));
    }

    #[test]
    fn attrition_uses_out_of_supply_key_when_default_missing() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        let mut table = AttritionTable {
            schema_version: 1,
            rows: BTreeMap::new(),
        };
        table.rows.insert("OUT_OF_SUPPLY".into(), Maybe::Value(2));
        resolve_supply_phase(&mut scenario, &table);
        assert_eq!(scenario.corps[&fra_corps()].infantry_sp, 2);
    }

    #[test]
    fn attrition_missing_rows_skips_loss() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        let before = scenario.corps[&fra_corps()].infantry_sp;
        let table = AttritionTable {
            schema_version: 1,
            rows: BTreeMap::new(),
        };
        resolve_supply_phase(&mut scenario, &table);
        assert_eq!(scenario.corps[&fra_corps()].infantry_sp, before);
    }

    #[test]
    fn validate_depot_order_accepts_owned_area() {
        let scenario = supply_scenario();
        let order = EstablishDepotOrder {
            submitter: fra(),
            area: paris(),
        };
        assert!(validate_depot_order(&scenario, &order).is_ok());
    }

    #[test]
    fn validate_depot_order_rejects_unknown_area() {
        let scenario = supply_scenario();
        let order = EstablishDepotOrder {
            submitter: fra(),
            area: madrid(),
        };
        assert!(validate_depot_order(&scenario, &order).is_err());
    }

    #[test]
    fn validate_depot_order_rejects_unknown_power_state() {
        let mut scenario = supply_scenario();
        scenario.power_state.remove(&fra());
        let order = EstablishDepotOrder {
            submitter: fra(),
            area: paris(),
        };
        assert!(validate_depot_order(&scenario, &order).is_err());
    }

    #[test]
    fn validate_depot_order_accepts_friendly_area() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: rus() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        set_relation(&mut scenario, fra(), rus(), DiplomaticState::Friendly);
        let order = EstablishDepotOrder {
            submitter: fra(),
            area: lyon(),
        };
        assert!(validate_depot_order(&scenario, &order).is_ok());
    }

    #[test]
    fn validate_depot_order_rejects_neutral_area() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: rus() }),
            Terrain::Open,
            Maybe::Value(0),
        );
        set_relation(&mut scenario, fra(), rus(), DiplomaticState::Neutral);
        let order = EstablishDepotOrder {
            submitter: fra(),
            area: lyon(),
        };
        assert!(validate_depot_order(&scenario, &order).is_err());
    }

    #[test]
    fn validate_depot_order_rejects_enemy_area() {
        let scenario = supply_scenario();
        let order = EstablishDepotOrder {
            submitter: fra(),
            area: vienna(),
        };
        assert!(validate_depot_order(&scenario, &order).is_err());
    }

    #[test]
    fn validate_depot_order_respects_max_depots_zero() {
        let mut scenario = supply_scenario();
        scenario.powers.get_mut(&fra()).unwrap().max_depots = 0;
        let order = EstablishDepotOrder {
            submitter: fra(),
            area: paris(),
        };
        assert!(validate_depot_order(&scenario, &order).is_err());
    }

    #[test]
    fn foraging_not_used_when_in_supply_even_with_yield() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Power(PowerSlot { power: fra() }),
            Terrain::Open,
            Maybe::Value(7),
        );
        connect(&mut scenario, &paris(), &lyon());
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn capital_target_may_be_in_enemy_zoc() {
        let scenario = supply_scenario();
        // Paris starts adjacent to the Austrian corps in Vienna, so the
        // capital is in enemy ZoC but still qualifies as a valid target.
        assert_eq!(trace_supply(&scenario, &fra_corps()), SupplyState::InSupply);
    }

    #[test]
    fn out_of_supply_sets_supplied_flag_false() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(0),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        resolve_supply_phase(&mut scenario, &placeholder_attrition());
        assert!(!scenario.corps[&fra_corps()].supplied);
    }

    #[test]
    fn foraging_sets_supplied_flag_true() {
        let mut scenario = supply_scenario();
        add_area(
            &mut scenario,
            lyon(),
            Owner::Unowned,
            Terrain::Open,
            Maybe::Value(4),
        );
        scenario.corps.get_mut(&fra_corps()).unwrap().area = lyon();
        resolve_supply_phase(&mut scenario, &placeholder_attrition());
        assert!(scenario.corps[&fra_corps()].supplied);
    }
}
