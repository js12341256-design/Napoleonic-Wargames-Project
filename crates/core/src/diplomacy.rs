//! Diplomatic order validation and Phase 6 resolution.
//!
//! Canonical rules reference: `docs/rules/diplomacy.md`.
//! HARD RULES (PROMPT.md §0): no invented numerics, no floats,
//! deterministic ordered iteration, and no `HashMap` in simulation logic.

use std::collections::BTreeMap;

use gc1805_core_schema::events::Event;
use gc1805_core_schema::ids::PowerId;
use gc1805_core_schema::scenario::{DiplomaticPairKey, DiplomaticState, Scenario};
use gc1805_core_schema::tables::{Maybe, PpModifiersTable};

use crate::orders::{
    BreakAllianceOrder, DeclareWarOrder, FormAllianceOrder, Order, ProposePeaceOrder,
};

/// Returns the current symmetric diplomatic state for a power pair.
/// Missing entries default to `Neutral`.
pub fn get_diplomatic_state(scenario: &Scenario, a: &PowerId, b: &PowerId) -> DiplomaticState {
    let key = DiplomaticPairKey::new(a.clone(), b.clone());
    scenario
        .diplomacy
        .get(&key)
        .copied()
        .unwrap_or(DiplomaticState::Neutral)
}

/// Inserts or updates the canonical diplomatic pair entry.
pub fn set_diplomatic_state(
    scenario: &mut Scenario,
    a: &PowerId,
    b: &PowerId,
    state: DiplomaticState,
) {
    let key = DiplomaticPairKey::new(a.clone(), b.clone());
    scenario.diplomacy.insert(key, state);
}

/// Pure validator for diplomacy-family orders.
pub fn validate_diplomatic_order(scenario: &Scenario, order: &Order) -> Result<(), String> {
    match order {
        Order::DeclareWar(o) => validate_declare_war(scenario, o),
        Order::ProposePeace(o) => validate_propose_peace(scenario, o),
        Order::FormAlliance(o) => validate_form_alliance(scenario, o),
        Order::BreakAlliance(o) => validate_break_alliance(scenario, o),
        _ => Err("not a diplomatic order".into()),
    }
}

/// Resolves the Phase 6 diplomacy steps implemented so far:
/// 1) declare war, 2) break alliance, 3) form alliance, 4) propose peace.
///
/// Steps 5-9 are reserved for future phases and intentionally no-op here.
pub fn resolve_diplomatic_phase(
    scenario: &mut Scenario,
    tables: &PpModifiersTable,
    orders: &[Order],
) -> Vec<Event> {
    let mut events = Vec::new();

    // 1. Wars first, submitter order deterministic by BTreeMap key.
    for order in collect_declare_war_orders(orders) {
        if validate_declare_war(scenario, order).is_err() {
            continue;
        }

        set_diplomatic_state(
            scenario,
            &order.submitter,
            &order.target,
            DiplomaticState::War,
        );
        events.push(Event::WarDeclared {
            by: order.submitter.clone(),
            against: order.target.clone(),
        });

        if let Some(delta) = prestige_delta(tables, "declare_war") {
            if let Some(power_state) = scenario.power_state.get_mut(&order.submitter) {
                power_state.prestige += delta;
            }
            events.push(Event::PrestigeChanged {
                power: order.submitter.clone(),
                delta,
                reason: "declare_war".into(),
            });
        }

        apply_alliance_cascade(scenario, &order.submitter, &order.target, &mut events);
    }

    // 2. Break alliances.
    for order in collect_break_alliance_orders(orders) {
        if validate_break_alliance(scenario, order).is_err() {
            continue;
        }
        set_diplomatic_state(
            scenario,
            &order.submitter,
            &order.target,
            DiplomaticState::Neutral,
        );
        events.push(Event::AllianceBroken {
            power_a: order.submitter.clone(),
            power_b: order.target.clone(),
        });
    }

    // 3. Form alliances after wars/breaks.
    for order in collect_form_alliance_orders(orders) {
        if validate_form_alliance(scenario, order).is_err() {
            continue;
        }
        set_diplomatic_state(
            scenario,
            &order.submitter,
            &order.target,
            DiplomaticState::Allied,
        );
        events.push(Event::AllianceFormed {
            power_a: order.submitter.clone(),
            power_b: order.target.clone(),
        });
    }

    // 4. Peace proposals are revealed now; acceptance is future work.
    for order in collect_propose_peace_orders(orders) {
        if validate_propose_peace(scenario, order).is_err() {
            continue;
        }
        events.push(Event::PeaceProposed {
            by: order.submitter.clone(),
            to: order.target.clone(),
        });
    }

    events
}

fn validate_declare_war(scenario: &Scenario, order: &DeclareWarOrder) -> Result<(), String> {
    if order.submitter == order.target {
        return Err("cannot declare war on self".into());
    }
    if !scenario.powers.contains_key(&order.submitter) {
        return Err(format!("unknown power `{}`", order.submitter));
    }
    if !scenario.powers.contains_key(&order.target) {
        return Err(format!("unknown power `{}`", order.target));
    }
    if get_diplomatic_state(scenario, &order.submitter, &order.target) == DiplomaticState::War {
        return Err("powers are already at war".into());
    }
    Ok(())
}

fn validate_propose_peace(scenario: &Scenario, order: &ProposePeaceOrder) -> Result<(), String> {
    if get_diplomatic_state(scenario, &order.submitter, &order.target) != DiplomaticState::War {
        return Err("peace may only be proposed between powers currently at war".into());
    }
    Ok(())
}

fn validate_form_alliance(scenario: &Scenario, order: &FormAllianceOrder) -> Result<(), String> {
    if get_diplomatic_state(scenario, &order.submitter, &order.target) == DiplomaticState::Allied {
        return Err("powers are already allied".into());
    }
    if get_diplomatic_state(scenario, &order.submitter, &order.target) == DiplomaticState::War {
        return Err("powers at war cannot form an alliance".into());
    }
    Ok(())
}

fn validate_break_alliance(scenario: &Scenario, order: &BreakAllianceOrder) -> Result<(), String> {
    if get_diplomatic_state(scenario, &order.submitter, &order.target) != DiplomaticState::Allied {
        return Err("powers are not currently allied".into());
    }
    Ok(())
}

fn prestige_delta(tables: &PpModifiersTable, key: &str) -> Option<i32> {
    match tables.events.get(key) {
        Some(Maybe::Value(value)) => Some(*value),
        _ => None,
    }
}

fn collect_declare_war_orders(orders: &[Order]) -> Vec<&DeclareWarOrder> {
    let mut grouped: BTreeMap<PowerId, Vec<&DeclareWarOrder>> = BTreeMap::new();
    for order in orders {
        if let Order::DeclareWar(war) = order {
            grouped.entry(war.submitter.clone()).or_default().push(war);
        }
    }
    grouped.into_values().flatten().collect()
}

fn collect_break_alliance_orders(orders: &[Order]) -> Vec<&BreakAllianceOrder> {
    let mut grouped: BTreeMap<PowerId, Vec<&BreakAllianceOrder>> = BTreeMap::new();
    for order in orders {
        if let Order::BreakAlliance(break_alliance) = order {
            grouped
                .entry(break_alliance.submitter.clone())
                .or_default()
                .push(break_alliance);
        }
    }
    grouped.into_values().flatten().collect()
}

fn collect_form_alliance_orders(orders: &[Order]) -> Vec<&FormAllianceOrder> {
    let mut grouped: BTreeMap<PowerId, Vec<&FormAllianceOrder>> = BTreeMap::new();
    for order in orders {
        if let Order::FormAlliance(form) = order {
            grouped
                .entry(form.submitter.clone())
                .or_default()
                .push(form);
        }
    }
    grouped.into_values().flatten().collect()
}

fn collect_propose_peace_orders(orders: &[Order]) -> Vec<&ProposePeaceOrder> {
    let mut grouped: BTreeMap<PowerId, Vec<&ProposePeaceOrder>> = BTreeMap::new();
    for order in orders {
        if let Order::ProposePeace(peace) = order {
            grouped
                .entry(peace.submitter.clone())
                .or_default()
                .push(peace);
        }
    }
    grouped.into_values().flatten().collect()
}

fn apply_alliance_cascade(
    scenario: &mut Scenario,
    attacker: &PowerId,
    initial_target: &PowerId,
    events: &mut Vec<Event>,
) {
    let mut frontier: Vec<PowerId> = vec![initial_target.clone()];
    let mut index = 0usize;

    while index < frontier.len() {
        let current = frontier[index].clone();
        index += 1;

        let allied_partners = allied_partners(scenario, &current);
        for ally in allied_partners {
            if ally == *attacker || ally == initial_target.clone() {
                continue;
            }
            if get_diplomatic_state(scenario, attacker, &ally) == DiplomaticState::War {
                continue;
            }

            set_diplomatic_state(scenario, attacker, &ally, DiplomaticState::War);
            events.push(Event::AllianceCascade {
                new_belligerent: ally.clone(),
                against: attacker.clone(),
                via_ally: current.clone(),
            });
            frontier.push(ally);
        }
    }
}

fn allied_partners(scenario: &Scenario, power: &PowerId) -> Vec<PowerId> {
    let mut partners = Vec::new();
    for (pair, state) in &scenario.diplomacy {
        if *state != DiplomaticState::Allied {
            continue;
        }
        if &pair.0 == power {
            partners.push(pair.1.clone());
        } else if &pair.1 == power {
            partners.push(pair.0.clone());
        }
    }
    partners
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gc1805_core_schema::events::Event;
    use gc1805_core_schema::ids::{AreaId, LeaderId};
    use gc1805_core_schema::scenario::{
        Area, DiplomaticPairKey, DiplomaticState, Features, GameDate, Leader, MovementRules, Owner,
        PowerSetup, PowerSlot, PowerState, Scenario, Terrain, SCHEMA_VERSION,
    };
    use gc1805_core_schema::tables::{Maybe, PlaceholderMarker, PpModifiersTable};

    use super::*;
    use crate::orders::{
        BreakAllianceOrder, DeclareWarOrder, FormAllianceOrder, Order, ProposePeaceOrder,
    };

    fn diplo_scenario() -> Scenario {
        let fra = PowerId::from("FRA");
        let gbr = PowerId::from("GBR");
        let aus = PowerId::from("AUS");

        let mut powers = BTreeMap::new();
        powers.insert(
            fra.clone(),
            power_setup("France", "LEADER_FRA", "AREA_PARIS", 10),
        );
        powers.insert(
            gbr.clone(),
            power_setup("Britain", "LEADER_GBR", "AREA_LONDON", 8),
        );
        powers.insert(
            aus.clone(),
            power_setup("Austria", "LEADER_AUS", "AREA_VIENNA", 7),
        );

        let mut power_state = BTreeMap::new();
        power_state.insert(fra.clone(), power_state_with_prestige(10));
        power_state.insert(gbr.clone(), power_state_with_prestige(8));
        power_state.insert(aus.clone(), power_state_with_prestige(7));

        let mut leaders = BTreeMap::new();
        leaders.insert(LeaderId::from("LEADER_FRA"), leader("French Ruler"));
        leaders.insert(LeaderId::from("LEADER_GBR"), leader("British Ruler"));
        leaders.insert(LeaderId::from("LEADER_AUS"), leader("Austrian Ruler"));

        let mut areas = BTreeMap::new();
        areas.insert(AreaId::from("AREA_PARIS"), capital_area("Paris", "FRA"));
        areas.insert(AreaId::from("AREA_LONDON"), capital_area("London", "GBR"));
        areas.insert(AreaId::from("AREA_VIENNA"), capital_area("Vienna", "AUS"));

        let mut diplomacy = BTreeMap::new();
        diplomacy.insert(
            DiplomaticPairKey::new(fra.clone(), gbr.clone()),
            DiplomaticState::War,
        );
        diplomacy.insert(
            DiplomaticPairKey::new(fra.clone(), aus.clone()),
            DiplomaticState::Neutral,
        );
        diplomacy.insert(
            DiplomaticPairKey::new(gbr.clone(), aus.clone()),
            DiplomaticState::Friendly,
        );

        Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "phase6_test".into(),
            name: "Diplomacy Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules::default(),
            current_turn: 0,
            power_state,
            production_queue: Vec::new(),
            replacement_queue: Vec::new(),
            subsidy_queue: Vec::new(),
            powers,
            minors: BTreeMap::new(),
            leaders,
            areas,
            sea_zones: BTreeMap::new(),
            corps: BTreeMap::new(),
            fleets: BTreeMap::new(),
            diplomacy,
            adjacency: Vec::new(),
            coast_links: Vec::new(),
            sea_adjacency: Vec::new(),
        }
    }

    fn power_setup(name: &str, ruler: &str, capital: &str, starting_pp: i32) -> PowerSetup {
        PowerSetup {
            display_name: name.into(),
            house: format!("House {name}"),
            ruler: LeaderId::from(ruler),
            capital: AreaId::from(capital),
            starting_treasury: 100,
            starting_manpower: 20,
            starting_pp,
            max_corps: 10,
            max_depots: 5,
            mobilization_areas: vec![AreaId::from(capital)],
            color_hex: "#000000".into(),
        }
    }

    fn power_state_with_prestige(prestige: i32) -> PowerState {
        PowerState {
            treasury: 100,
            manpower: 20,
            prestige,
            tax_policy: gc1805_core_schema::scenario::TaxPolicy::Standard,
        }
    }

    fn leader(name: &str) -> Leader {
        Leader {
            display_name: name.into(),
            strategic: 3,
            tactical: 3,
            initiative: 3,
            army_commander: true,
            born: GameDate::new(1760, 1),
        }
    }

    fn capital_area(name: &str, power: &str) -> Area {
        Area {
            display_name: name.into(),
            owner: Owner::Power(PowerSlot {
                power: PowerId::from(power),
            }),
            terrain: Terrain::Urban,
            fort_level: 2,
            money_yield: Maybe::Placeholder(PlaceholderMarker::new()),
            manpower_yield: Maybe::Placeholder(PlaceholderMarker::new()),
            capital_of: Some(PowerId::from(power)),
            port: false,
            blockaded: false,
            map_x: 0,
            map_y: 0,
        }
    }

    fn pp_tables_placeholder() -> PpModifiersTable {
        PpModifiersTable {
            schema_version: 1,
            events: BTreeMap::from([(
                "declare_war".into(),
                Maybe::Placeholder(PlaceholderMarker::new()),
            )]),
        }
    }

    fn add_prussia(s: &mut Scenario) {
        s.powers.insert(
            PowerId::from("PRU"),
            power_setup("Prussia", "LEADER_PRU", "AREA_BERLIN", 6),
        );
        s.power_state
            .insert(PowerId::from("PRU"), power_state_with_prestige(6));
        s.leaders
            .insert(LeaderId::from("LEADER_PRU"), leader("Prussian Ruler"));
        s.areas
            .insert(AreaId::from("AREA_BERLIN"), capital_area("Berlin", "PRU"));
    }

    #[test]
    fn get_state_war() {
        let s = diplo_scenario();
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("FRA"), &PowerId::from("GBR")),
            DiplomaticState::War
        );
    }

    #[test]
    fn get_state_neutral_when_missing() {
        let s = diplo_scenario();
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("FRA"), &PowerId::from("PRU")),
            DiplomaticState::Neutral
        );
    }

    #[test]
    fn get_state_symmetric() {
        let s = diplo_scenario();
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("FRA"), &PowerId::from("GBR")),
            get_diplomatic_state(&s, &PowerId::from("GBR"), &PowerId::from("FRA"))
        );
    }

    #[test]
    fn validate_declare_war_ok() {
        let s = diplo_scenario();
        let order = Order::DeclareWar(DeclareWarOrder {
            submitter: PowerId::from("AUS"),
            target: PowerId::from("FRA"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_ok());
    }

    #[test]
    fn validate_declare_war_already_at_war() {
        let s = diplo_scenario();
        let order = Order::DeclareWar(DeclareWarOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("GBR"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_err());
    }

    #[test]
    fn validate_declare_war_self() {
        let s = diplo_scenario();
        let order = Order::DeclareWar(DeclareWarOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("FRA"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_err());
    }

    #[test]
    fn validate_peace_ok() {
        let s = diplo_scenario();
        let order = Order::ProposePeace(ProposePeaceOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("GBR"),
            terms: "status quo".into(),
        });
        assert!(validate_diplomatic_order(&s, &order).is_ok());
    }

    #[test]
    fn validate_peace_not_at_war() {
        let s = diplo_scenario();
        let order = Order::ProposePeace(ProposePeaceOrder {
            submitter: PowerId::from("GBR"),
            target: PowerId::from("AUS"),
            terms: "white peace".into(),
        });
        assert!(validate_diplomatic_order(&s, &order).is_err());
    }

    #[test]
    fn validate_form_alliance_ok() {
        let s = diplo_scenario();
        let order = Order::FormAlliance(FormAllianceOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("AUS"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_ok());
    }

    #[test]
    fn validate_form_alliance_already_allied() {
        let mut s = diplo_scenario();
        set_diplomatic_state(
            &mut s,
            &PowerId::from("FRA"),
            &PowerId::from("AUS"),
            DiplomaticState::Allied,
        );
        let order = Order::FormAlliance(FormAllianceOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("AUS"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_err());
    }

    #[test]
    fn validate_form_alliance_at_war() {
        let s = diplo_scenario();
        let order = Order::FormAlliance(FormAllianceOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("GBR"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_err());
    }

    #[test]
    fn validate_break_alliance_ok() {
        let mut s = diplo_scenario();
        set_diplomatic_state(
            &mut s,
            &PowerId::from("FRA"),
            &PowerId::from("AUS"),
            DiplomaticState::Allied,
        );
        let order = Order::BreakAlliance(BreakAllianceOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("AUS"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_ok());
    }

    #[test]
    fn validate_break_alliance_not_allied() {
        let s = diplo_scenario();
        let order = Order::BreakAlliance(BreakAllianceOrder {
            submitter: PowerId::from("FRA"),
            target: PowerId::from("AUS"),
        });
        assert!(validate_diplomatic_order(&s, &order).is_err());
    }

    #[test]
    fn resolve_declare_war_sets_state() {
        let mut s = diplo_scenario();
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("AUS"),
                target: PowerId::from("FRA"),
            })],
        );
        assert!(!events.is_empty());
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("AUS"), &PowerId::from("FRA")),
            DiplomaticState::War
        );
    }

    #[test]
    fn resolve_declare_war_emits_event() {
        let mut s = diplo_scenario();
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("AUS"),
                target: PowerId::from("FRA"),
            })],
        );
        assert_eq!(
            events[0],
            Event::WarDeclared {
                by: PowerId::from("AUS"),
                against: PowerId::from("FRA")
            }
        );
    }

    #[test]
    fn resolve_declare_war_pp_change_placeholder_skipped() {
        let mut s = diplo_scenario();
        let starting_pp = s.power_state.get(&PowerId::from("AUS")).unwrap().prestige;
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("AUS"),
                target: PowerId::from("FRA"),
            })],
        );
        assert_eq!(
            s.power_state.get(&PowerId::from("AUS")).unwrap().prestige,
            starting_pp
        );
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::PrestigeChanged { .. })));
    }

    #[test]
    fn resolve_declare_war_pp_change_value_applied() {
        let mut s = diplo_scenario();
        let tables = PpModifiersTable {
            schema_version: 1,
            events: BTreeMap::from([("declare_war".into(), Maybe::Value(-3))]),
        };
        let events = resolve_diplomatic_phase(
            &mut s,
            &tables,
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("AUS"),
                target: PowerId::from("FRA"),
            })],
        );
        assert_eq!(
            s.power_state.get(&PowerId::from("AUS")).unwrap().prestige,
            4
        );
        assert!(events.iter().any(|event| matches!(
            event,
            Event::PrestigeChanged {
                power,
                delta: -3,
                reason
            } if *power == PowerId::from("AUS") && reason == "declare_war"
        )));
    }

    #[test]
    fn resolve_form_alliance_sets_state() {
        let mut s = diplo_scenario();
        resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::FormAlliance(FormAllianceOrder {
                submitter: PowerId::from("FRA"),
                target: PowerId::from("AUS"),
            })],
        );
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("FRA"), &PowerId::from("AUS")),
            DiplomaticState::Allied
        );
    }

    #[test]
    fn resolve_form_alliance_emits_event() {
        let mut s = diplo_scenario();
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::FormAlliance(FormAllianceOrder {
                submitter: PowerId::from("FRA"),
                target: PowerId::from("AUS"),
            })],
        );
        assert!(events.iter().any(|event| matches!(
            event,
            Event::AllianceFormed { power_a, power_b }
                if *power_a == PowerId::from("FRA") && *power_b == PowerId::from("AUS")
        )));
    }

    #[test]
    fn resolve_break_alliance_sets_state() {
        let mut s = diplo_scenario();
        set_diplomatic_state(
            &mut s,
            &PowerId::from("FRA"),
            &PowerId::from("AUS"),
            DiplomaticState::Allied,
        );
        resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::BreakAlliance(BreakAllianceOrder {
                submitter: PowerId::from("FRA"),
                target: PowerId::from("AUS"),
            })],
        );
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("FRA"), &PowerId::from("AUS")),
            DiplomaticState::Neutral
        );
    }

    #[test]
    fn resolve_peace_proposed_emits_event() {
        let mut s = diplo_scenario();
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::ProposePeace(ProposePeaceOrder {
                submitter: PowerId::from("FRA"),
                target: PowerId::from("GBR"),
                terms: "status quo".into(),
            })],
        );
        assert!(events.iter().any(|event| matches!(
            event,
            Event::PeaceProposed { by, to }
                if *by == PowerId::from("FRA") && *to == PowerId::from("GBR")
        )));
    }

    #[test]
    fn alliance_cascade_triggered() {
        let mut s = diplo_scenario();
        add_prussia(&mut s);
        set_diplomatic_state(
            &mut s,
            &PowerId::from("FRA"),
            &PowerId::from("AUS"),
            DiplomaticState::Allied,
        );
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("PRU"),
                target: PowerId::from("FRA"),
            })],
        );
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("PRU"), &PowerId::from("AUS")),
            DiplomaticState::War
        );
        assert!(events.iter().any(|event| matches!(
            event,
            Event::AllianceCascade {
                new_belligerent,
                against,
                via_ally
            } if *new_belligerent == PowerId::from("AUS")
                && *against == PowerId::from("PRU")
                && *via_ally == PowerId::from("FRA")
        )));
    }

    #[test]
    fn alliance_cascade_not_triggered_if_already_at_war() {
        let mut s = diplo_scenario();
        set_diplomatic_state(
            &mut s,
            &PowerId::from("FRA"),
            &PowerId::from("AUS"),
            DiplomaticState::Allied,
        );
        set_diplomatic_state(
            &mut s,
            &PowerId::from("GBR"),
            &PowerId::from("AUS"),
            DiplomaticState::War,
        );
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("GBR"),
                target: PowerId::from("FRA"),
            })],
        );
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::AllianceCascade { .. })));
    }

    #[test]
    fn alliance_cascade_not_triggered_if_no_alliance() {
        let mut s = diplo_scenario();
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("AUS"),
                target: PowerId::from("GBR"),
            })],
        );
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("AUS"), &PowerId::from("FRA")),
            DiplomaticState::Neutral
        );
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::AllianceCascade { .. })));
    }

    #[test]
    fn resolve_multiple_wars_deterministic_order() {
        let mut s = diplo_scenario();
        add_prussia(&mut s);

        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[
                Order::DeclareWar(DeclareWarOrder {
                    submitter: PowerId::from("PRU"),
                    target: PowerId::from("AUS"),
                }),
                Order::DeclareWar(DeclareWarOrder {
                    submitter: PowerId::from("AUS"),
                    target: PowerId::from("FRA"),
                }),
            ],
        );

        let war_events: Vec<(PowerId, PowerId)> = events
            .into_iter()
            .filter_map(|event| match event {
                Event::WarDeclared { by, against } => Some((by, against)),
                _ => None,
            })
            .collect();
        assert_eq!(
            war_events,
            vec![
                (PowerId::from("AUS"), PowerId::from("FRA")),
                (PowerId::from("PRU"), PowerId::from("AUS")),
            ]
        );
    }

    #[test]
    fn resolve_order_wars_before_alliances() {
        let mut s = diplo_scenario();
        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[
                Order::FormAlliance(FormAllianceOrder {
                    submitter: PowerId::from("AUS"),
                    target: PowerId::from("FRA"),
                }),
                Order::DeclareWar(DeclareWarOrder {
                    submitter: PowerId::from("AUS"),
                    target: PowerId::from("FRA"),
                }),
            ],
        );
        assert!(matches!(events.first(), Some(Event::WarDeclared { .. })));
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("AUS"), &PowerId::from("FRA")),
            DiplomaticState::War
        );
    }

    #[test]
    fn set_state_canonical_key() {
        let mut s = diplo_scenario();
        set_diplomatic_state(
            &mut s,
            &PowerId::from("GBR"),
            &PowerId::from("FRA"),
            DiplomaticState::Friendly,
        );
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("FRA"), &PowerId::from("GBR")),
            DiplomaticState::Friendly
        );
        assert!(s.diplomacy.contains_key(&DiplomaticPairKey::new(
            PowerId::from("FRA"),
            PowerId::from("GBR")
        )));
    }

    #[test]
    fn resolve_empty_orders() {
        let mut s = diplo_scenario();
        let events = resolve_diplomatic_phase(&mut s, &pp_tables_placeholder(), &[]);
        assert!(events.is_empty());
    }

    #[test]
    fn resolve_break_then_form_same_turn() {
        let mut s = diplo_scenario();
        set_diplomatic_state(
            &mut s,
            &PowerId::from("FRA"),
            &PowerId::from("AUS"),
            DiplomaticState::Allied,
        );
        resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[
                Order::FormAlliance(FormAllianceOrder {
                    submitter: PowerId::from("FRA"),
                    target: PowerId::from("AUS"),
                }),
                Order::BreakAlliance(BreakAllianceOrder {
                    submitter: PowerId::from("FRA"),
                    target: PowerId::from("AUS"),
                }),
            ],
        );
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("FRA"), &PowerId::from("AUS")),
            DiplomaticState::Allied
        );
    }

    #[test]
    fn resolve_declare_war_cascade_chain() {
        let mut s = diplo_scenario();
        add_prussia(&mut s);
        s.powers.insert(
            PowerId::from("RUS"),
            power_setup("Russia", "LEADER_RUS", "AREA_MOSCOW", 9),
        );
        s.power_state
            .insert(PowerId::from("RUS"), power_state_with_prestige(9));
        s.leaders
            .insert(LeaderId::from("LEADER_RUS"), leader("Russian Ruler"));
        s.areas
            .insert(AreaId::from("AREA_MOSCOW"), capital_area("Moscow", "RUS"));

        set_diplomatic_state(
            &mut s,
            &PowerId::from("FRA"),
            &PowerId::from("AUS"),
            DiplomaticState::Allied,
        );
        set_diplomatic_state(
            &mut s,
            &PowerId::from("AUS"),
            &PowerId::from("PRU"),
            DiplomaticState::Allied,
        );

        let events = resolve_diplomatic_phase(
            &mut s,
            &pp_tables_placeholder(),
            &[Order::DeclareWar(DeclareWarOrder {
                submitter: PowerId::from("RUS"),
                target: PowerId::from("FRA"),
            })],
        );

        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("RUS"), &PowerId::from("AUS")),
            DiplomaticState::War
        );
        assert_eq!(
            get_diplomatic_state(&s, &PowerId::from("RUS"), &PowerId::from("PRU")),
            DiplomaticState::War
        );
        assert!(events.iter().any(|event| matches!(
            event,
            Event::AllianceCascade {
                new_belligerent,
                against,
                via_ally
            } if *new_belligerent == PowerId::from("PRU")
                && *against == PowerId::from("RUS")
                && *via_ally == PowerId::from("AUS")
        )));
    }
}
