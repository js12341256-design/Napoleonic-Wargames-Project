//! Naval movement, combat, blockade, and transport (PROMPT.md §16.9).
//!
//! HARD RULES:
//! - No floats.
//! - No hash-ordered iteration.
//! - Placeholder tables reject resolution instead of inventing values.

use std::collections::{BTreeMap, BTreeSet};

use gc1805_core_schema::{
    events::{Event, OrderRejected},
    ids::{FleetId, PowerId, SeaZoneId},
    naval_types::NavalOutcome,
    scenario::{DiplomaticPairKey, DiplomaticState, Owner, Scenario},
    tables::{Maybe, NavalCombatTable},
};

use crate::orders::{DisembarkOrder, EmbarkOrder, MoveFleetOrder, NavalAttackOrder};

#[derive(Debug, Clone, Default)]
pub struct SeaGraph {
    neighbours: BTreeMap<SeaZoneId, BTreeSet<SeaZoneId>>,
}

impl SeaGraph {
    pub fn new(scenario: &Scenario) -> Self {
        let mut neighbours: BTreeMap<SeaZoneId, BTreeSet<SeaZoneId>> = BTreeMap::new();
        for zone in scenario.sea_zones.keys() {
            neighbours.insert(zone.clone(), BTreeSet::new());
        }
        for edge in &scenario.sea_adjacency {
            neighbours
                .entry(edge.from.clone())
                .or_default()
                .insert(edge.to.clone());
            neighbours
                .entry(edge.to.clone())
                .or_default()
                .insert(edge.from.clone());
        }
        Self { neighbours }
    }

    pub fn adjacent_zones(&self, zone: &SeaZoneId) -> Vec<SeaZoneId> {
        self.neighbours
            .get(zone)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn shortest_path_hops(&self, from: &SeaZoneId, to: &SeaZoneId) -> Option<usize> {
        if !self.neighbours.contains_key(from) || !self.neighbours.contains_key(to) {
            return None;
        }
        if from == to {
            return Some(0);
        }

        let mut visited: BTreeSet<SeaZoneId> = BTreeSet::new();
        let mut frontier: Vec<(SeaZoneId, usize)> = vec![(from.clone(), 0)];
        visited.insert(from.clone());

        while !frontier.is_empty() {
            let mut next: Vec<(SeaZoneId, usize)> = Vec::new();
            for (zone, hops) in &frontier {
                if let Some(nbrs) = self.neighbours.get(zone) {
                    for nbr in nbrs {
                        if visited.insert(nbr.clone()) {
                            if nbr == to {
                                return Some(*hops + 1);
                            }
                            next.push((nbr.clone(), *hops + 1));
                        }
                    }
                }
            }
            frontier = next;
        }

        None
    }
}

pub fn validate_fleet_move(
    scenario: &Scenario,
    graph: &SeaGraph,
    order: &MoveFleetOrder,
) -> Result<(), String> {
    let fleet = scenario
        .fleets
        .get(&order.fleet)
        .ok_or_else(|| format!("unknown fleet `{}`", order.fleet))?;
    if fleet.owner != order.submitter {
        return Err(format!(
            "fleet `{}` is owned by `{}`, not `{}`",
            order.fleet, fleet.owner, order.submitter
        ));
    }
    let from = fleet
        .at_sea
        .as_ref()
        .ok_or_else(|| format!("fleet `{}` is not at sea", order.fleet))?;
    if !scenario.sea_zones.contains_key(&order.destination) {
        return Err(format!(
            "unknown destination sea zone `{}`",
            order.destination
        ));
    }
    if graph.shortest_path_hops(from, &order.destination).is_none() {
        return Err(format!(
            "no sea path from `{}` to `{}`",
            from, order.destination
        ));
    }
    Ok(())
}

pub fn resolve_fleet_move(
    scenario: &mut Scenario,
    graph: &SeaGraph,
    order: &MoveFleetOrder,
) -> Vec<Event> {
    debug_assert!(validate_fleet_move(scenario, graph, order).is_ok());

    let from = scenario
        .fleets
        .get(&order.fleet)
        .and_then(|fleet| fleet.at_sea.clone())
        .expect("validated fleet move must start at sea");

    if let Some(fleet) = scenario.fleets.get_mut(&order.fleet) {
        fleet.at_sea = Some(order.destination.clone());
        fleet.at_port = None;
    }

    let mut events = vec![Event::FleetMoved {
        fleet: order.fleet.clone(),
        from,
        to: order.destination.clone(),
    }];

    let owner = scenario
        .fleets
        .get(&order.fleet)
        .map(|fleet| fleet.owner.clone())
        .expect("fleet must exist after validated move");

    let mut emitted_blockade = false;
    for link in &scenario.coast_links {
        if link.sea != order.destination {
            continue;
        }
        let Some(area) = scenario.areas.get_mut(&link.area) else {
            continue;
        };
        if !area.port {
            continue;
        }
        let enemy_owned = matches!(&area.owner, Owner::Power(slot) if slot.power != owner);
        if enemy_owned {
            area.blockaded = true;
            if !emitted_blockade {
                events.push(Event::BlockadeEstablished {
                    fleet: order.fleet.clone(),
                    sea_zone: order.destination.clone(),
                });
                emitted_blockade = true;
            }
        }
    }

    events
}

pub fn resolve_naval_battle(
    scenario: &mut Scenario,
    tables: &NavalCombatTable,
    rng_seed: u64,
    order: &NavalAttackOrder,
) -> Vec<Event> {
    let Some(attacking_fleet) = scenario.fleets.get(&order.fleet) else {
        return vec![Event::OrderRejected(OrderRejected {
            reason_code: "UNKNOWN_FLEET".into(),
            message: format!("unknown fleet `{}`", order.fleet),
        })];
    };
    if attacking_fleet.owner != order.submitter {
        return vec![Event::OrderRejected(OrderRejected {
            reason_code: "FLEET_NOT_OWNER".into(),
            message: format!(
                "fleet `{}` is owned by `{}`, not `{}`",
                order.fleet, attacking_fleet.owner, order.submitter
            ),
        })];
    }

    let attacker_zone = attacking_fleet.at_sea.clone();
    if attacker_zone.as_ref() != Some(&order.target_zone) {
        return vec![Event::OrderRejected(OrderRejected {
            reason_code: "FLEET_NOT_IN_TARGET_ZONE".into(),
            message: format!(
                "fleet `{}` is not in target zone `{}`",
                order.fleet, order.target_zone
            ),
        })];
    }

    let defender_ids: Vec<FleetId> = scenario
        .fleets
        .iter()
        .filter_map(|(fleet_id, fleet)| {
            if fleet.owner == order.submitter || fleet.at_sea.as_ref() != Some(&order.target_zone) {
                return None;
            }
            let pair = DiplomaticPairKey::new(order.submitter.clone(), fleet.owner.clone());
            match scenario.diplomacy.get(&pair) {
                Some(DiplomaticState::War) => Some(fleet_id.clone()),
                _ => None,
            }
        })
        .collect();

    if defender_ids.is_empty() {
        return vec![Event::OrderRejected(OrderRejected {
            reason_code: "NO_NAVAL_DEFENDER".into(),
            message: format!("no enemy fleet in `{}`", order.target_zone),
        })];
    }

    let attacker_ships_before = scenario
        .fleets
        .get(&order.fleet)
        .map(total_ships)
        .unwrap_or(0);
    let defender_ships_before: i32 = defender_ids
        .iter()
        .filter_map(|id| scenario.fleets.get(id))
        .map(total_ships)
        .sum();

    let bucket = naval_ratio_bucket(attacker_ships_before, defender_ships_before);
    let Some(result_row) = tables.results.get(bucket) else {
        return vec![Event::OrderRejected(OrderRejected {
            reason_code: "NAVAL_TABLE_PLACEHOLDER".into(),
            message: format!("naval combat bucket `{bucket}` missing or placeholder"),
        })];
    };
    let die_index = (rng_seed % tables.die_faces as u64) as usize;
    let Some(result_entry) = result_row.get(die_index) else {
        return vec![Event::OrderRejected(OrderRejected {
            reason_code: "NAVAL_TABLE_PLACEHOLDER".into(),
            message: "naval combat die result missing or placeholder".into(),
        })];
    };
    let result = match result_entry {
        Maybe::Value(value) => value.clone(),
        Maybe::Placeholder(_) => {
            return vec![Event::OrderRejected(OrderRejected {
                reason_code: "NAVAL_TABLE_PLACEHOLDER".into(),
                message: "naval combat values are PLACEHOLDER".into(),
            })]
        }
    };

    apply_ship_loss(
        scenario,
        &order.fleet,
        result.attacker_ship_loss.min(attacker_ships_before),
    );
    distribute_ship_loss(
        scenario,
        &defender_ids,
        result.defender_ship_loss.min(defender_ships_before),
    );

    let attacker_after = scenario
        .fleets
        .get(&order.fleet)
        .map(total_ships)
        .unwrap_or(0);
    let defender_after: i32 = defender_ids
        .iter()
        .filter_map(|id| scenario.fleets.get(id))
        .map(total_ships)
        .sum();

    let defender = scenario
        .fleets
        .get(&defender_ids[0])
        .map(|fleet| fleet.owner.clone())
        .unwrap_or_else(|| PowerId::from("UNKNOWN"));

    let outcome = if defender_after == 0 {
        NavalOutcome::DefenderSunk
    } else if attacker_after < attacker_ships_before && defender_after < defender_ships_before {
        NavalOutcome::MutualLoss
    } else {
        NavalOutcome::AttackerRepulsed
    };

    vec![Event::NavalBattleResolved {
        sea_zone: order.target_zone.clone(),
        attacker: order.submitter.clone(),
        defender,
        attacker_ships_lost: attacker_ships_before - attacker_after,
        defender_ships_lost: defender_ships_before - defender_after,
        outcome,
    }]
}

pub fn validate_embark(scenario: &Scenario, order: &EmbarkOrder) -> Result<(), String> {
    let corps = scenario
        .corps
        .get(&order.corps)
        .ok_or_else(|| format!("unknown corps `{}`", order.corps))?;
    if corps.owner != order.submitter {
        return Err(format!(
            "corps `{}` is owned by `{}`, not `{}`",
            order.corps, corps.owner, order.submitter
        ));
    }

    let fleet = scenario
        .fleets
        .get(&order.fleet)
        .ok_or_else(|| format!("unknown fleet `{}`", order.fleet))?;
    if fleet.owner != order.submitter {
        return Err(format!(
            "fleet `{}` is owned by `{}`, not `{}`",
            order.fleet, fleet.owner, order.submitter
        ));
    }
    let fleet_zone = fleet
        .at_sea
        .as_ref()
        .ok_or_else(|| format!("fleet `{}` is not at sea", order.fleet))?;

    let area = scenario
        .areas
        .get(&corps.area)
        .ok_or_else(|| format!("unknown corps area `{}`", corps.area))?;
    if !area.port {
        return Err(format!("area `{}` is not a port", corps.area));
    }

    let linked = scenario
        .coast_links
        .iter()
        .any(|link| link.area == corps.area && &link.sea == fleet_zone);
    if !linked {
        return Err(format!(
            "area `{}` has no coast link to fleet zone `{}`",
            corps.area, fleet_zone
        ));
    }
    if fleet.embarked_corps.contains(&order.corps) {
        return Err(format!("corps `{}` already embarked", order.corps));
    }
    Ok(())
}

pub fn resolve_embark(scenario: &mut Scenario, order: &EmbarkOrder) -> Event {
    debug_assert!(validate_embark(scenario, order).is_ok());
    let area = scenario
        .corps
        .get(&order.corps)
        .map(|corps| corps.area.clone())
        .expect("validated embark must have corps");
    if let Some(fleet) = scenario.fleets.get_mut(&order.fleet) {
        fleet.embarked_corps.push(order.corps.clone());
        fleet.embarked_corps.sort();
    }
    Event::CorpsEmbarked {
        corps: order.corps.clone(),
        fleet: order.fleet.clone(),
        area,
    }
}

pub fn validate_disembark(scenario: &Scenario, order: &DisembarkOrder) -> Result<(), String> {
    let fleet = scenario
        .fleets
        .get(&order.fleet)
        .ok_or_else(|| format!("unknown fleet `{}`", order.fleet))?;
    if fleet.owner != order.submitter {
        return Err(format!(
            "fleet `{}` is owned by `{}`, not `{}`",
            order.fleet, fleet.owner, order.submitter
        ));
    }
    let fleet_zone = fleet
        .at_sea
        .as_ref()
        .ok_or_else(|| format!("fleet `{}` is not at sea", order.fleet))?;
    if !fleet.embarked_corps.contains(&order.corps) {
        return Err(format!(
            "corps `{}` is not embarked on fleet `{}`",
            order.corps, order.fleet
        ));
    }
    let area = scenario
        .areas
        .get(&order.target_area)
        .ok_or_else(|| format!("unknown target area `{}`", order.target_area))?;
    if !area.port {
        return Err(format!("area `{}` is not a port", order.target_area));
    }
    let linked = scenario
        .coast_links
        .iter()
        .any(|link| link.area == order.target_area && &link.sea == fleet_zone);
    if !linked {
        return Err(format!(
            "target area `{}` has no coast link to fleet zone `{}`",
            order.target_area, fleet_zone
        ));
    }
    Ok(())
}

pub fn resolve_disembark(scenario: &mut Scenario, order: &DisembarkOrder) -> Event {
    debug_assert!(validate_disembark(scenario, order).is_ok());
    if let Some(corps) = scenario.corps.get_mut(&order.corps) {
        corps.area = order.target_area.clone();
    }
    if let Some(fleet) = scenario.fleets.get_mut(&order.fleet) {
        fleet
            .embarked_corps
            .retain(|corp_id| corp_id != &order.corps);
    }
    Event::CorpsDisembarked {
        corps: order.corps.clone(),
        fleet: order.fleet.clone(),
        area: order.target_area.clone(),
    }
}

fn total_ships(fleet: &gc1805_core_schema::scenario::Fleet) -> i32 {
    fleet.ships_of_the_line + fleet.frigates + fleet.transports
}

fn apply_ship_loss(scenario: &mut Scenario, fleet_id: &FleetId, mut loss: i32) {
    let Some(fleet) = scenario.fleets.get_mut(fleet_id) else {
        return;
    };
    let step = fleet.ships_of_the_line.min(loss);
    fleet.ships_of_the_line -= step;
    loss -= step;
    if loss > 0 {
        let step = fleet.frigates.min(loss);
        fleet.frigates -= step;
        loss -= step;
    }
    if loss > 0 {
        let step = fleet.transports.min(loss);
        fleet.transports -= step;
    }
}

fn distribute_ship_loss(scenario: &mut Scenario, fleets: &[FleetId], total_loss: i32) {
    if fleets.is_empty() || total_loss <= 0 {
        return;
    }
    let fleet_count = fleets.len() as i32;
    let base = total_loss / fleet_count;
    let remainder = total_loss % fleet_count;
    for (index, fleet_id) in fleets.iter().enumerate() {
        let extra = if (index as i32) < remainder { 1 } else { 0 };
        apply_ship_loss(scenario, fleet_id, base + extra);
    }
}

fn naval_ratio_bucket(attacker: i32, defender: i32) -> &'static str {
    if defender <= 0 || attacker >= 2 * defender {
        "2:1"
    } else if attacker >= defender {
        "1:1"
    } else {
        "1:2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::{
        ids::{AreaId, CorpsId, FleetId, LeaderId, PowerId, SeaZoneId},
        scenario::{
            Area, CoastLink, Corps, DiplomaticState, Features, Fleet, GameDate, Leader,
            MovementRules, Owner, PowerSetup, PowerSlot, Scenario, SeaAdjacency, SeaZone, Terrain,
            SCHEMA_VERSION,
        },
        tables::{Maybe, NavalCombatTable, NavalResult},
    };

    fn naval_scenario() -> Scenario {
        let mut scenario = Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "naval".into(),
            name: "naval".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1805, 5),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules::default(),
            current_turn: 0,
            power_state: BTreeMap::new(),
            production_queue: Vec::new(),
            replacement_queue: Vec::new(),
            subsidy_queue: Vec::new(),
            powers: BTreeMap::new(),
            minors: BTreeMap::new(),
            leaders: BTreeMap::new(),
            areas: BTreeMap::new(),
            sea_zones: BTreeMap::new(),
            corps: BTreeMap::new(),
            fleets: BTreeMap::new(),
            diplomacy: BTreeMap::new(),
            adjacency: Vec::new(),
            coast_links: Vec::new(),
            sea_adjacency: Vec::new(),
        };

        scenario.leaders.insert(
            LeaderId::from("LEADER_N"),
            Leader {
                display_name: "Napoleon".into(),
                strategic: 5,
                tactical: 5,
                initiative: 6,
                army_commander: true,
                born: GameDate::new(1769, 8),
            },
        );

        scenario.powers.insert(
            PowerId::from("FRA"),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: LeaderId::from("LEADER_N"),
                capital: AreaId::from("AREA_PARIS"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 5,
                mobilization_areas: vec![AreaId::from("AREA_PARIS")],
                color_hex: "#0000ff".into(),
            },
        );
        scenario.powers.insert(
            PowerId::from("GBR"),
            PowerSetup {
                display_name: "Britain".into(),
                house: "Hanover".into(),
                ruler: LeaderId::from("LEADER_N"),
                capital: AreaId::from("AREA_LONDON"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 5,
                mobilization_areas: vec![AreaId::from("AREA_LONDON")],
                color_hex: "#ff0000".into(),
            },
        );

        scenario.areas.insert(
            AreaId::from("AREA_PARIS"),
            Area {
                display_name: "Paris".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("FRA"),
                }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: Maybe::Value(0),
                manpower_yield: Maybe::Value(0),
                capital_of: None,
                port: true,
                blockaded: false,
                map_x: 0,
                map_y: 0,
            },
        );
        scenario.areas.insert(
            AreaId::from("AREA_LONDON"),
            Area {
                display_name: "London".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("GBR"),
                }),
                terrain: Terrain::Urban,
                fort_level: 1,
                money_yield: Maybe::Value(0),
                manpower_yield: Maybe::Value(0),
                capital_of: None,
                port: true,
                blockaded: false,
                map_x: 1,
                map_y: 1,
            },
        );
        scenario.areas.insert(
            AreaId::from("AREA_BREST"),
            Area {
                display_name: "Brest".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("FRA"),
                }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: Maybe::Value(0),
                manpower_yield: Maybe::Value(0),
                capital_of: None,
                port: true,
                blockaded: false,
                map_x: 2,
                map_y: 2,
            },
        );
        scenario.areas.insert(
            AreaId::from("AREA_INLAND"),
            Area {
                display_name: "Inland".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("FRA"),
                }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: Maybe::Value(0),
                manpower_yield: Maybe::Value(0),
                capital_of: None,
                port: false,
                blockaded: false,
                map_x: 3,
                map_y: 3,
            },
        );

        scenario.sea_zones.insert(
            SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
            SeaZone {
                display_name: "English Channel".into(),
                map_x: 0,
                map_y: 0,
            },
        );
        scenario.sea_zones.insert(
            SeaZoneId::from("SEA_NORTH_SEA"),
            SeaZone {
                display_name: "North Sea".into(),
                map_x: 1,
                map_y: 0,
            },
        );
        scenario.sea_zones.insert(
            SeaZoneId::from("SEA_IRISH_SEA"),
            SeaZone {
                display_name: "Irish Sea".into(),
                map_x: 2,
                map_y: 0,
            },
        );
        scenario.sea_zones.insert(
            SeaZoneId::from("SEA_BALTIC"),
            SeaZone {
                display_name: "Baltic".into(),
                map_x: 3,
                map_y: 0,
            },
        );

        scenario.corps.insert(
            CorpsId::from("CORPS_FRA_001"),
            Corps {
                display_name: "I Corps".into(),
                owner: PowerId::from("FRA"),
                area: AreaId::from("AREA_PARIS"),
                infantry_sp: 8,
                cavalry_sp: 2,
                artillery_sp: 1,
                morale_q4: 8000,
                supplied: true,
                leader: None,
            },
        );
        scenario.corps.insert(
            CorpsId::from("CORPS_GBR_001"),
            Corps {
                display_name: "British Corps".into(),
                owner: PowerId::from("GBR"),
                area: AreaId::from("AREA_LONDON"),
                infantry_sp: 6,
                cavalry_sp: 1,
                artillery_sp: 1,
                morale_q4: 8000,
                supplied: true,
                leader: None,
            },
        );

        scenario.fleets.insert(
            FleetId::from("FLEET_FRA_001"),
            Fleet {
                display_name: "French Fleet".into(),
                owner: PowerId::from("FRA"),
                at_port: None,
                at_sea: Some(SeaZoneId::from("SEA_ENGLISH_CHANNEL")),
                ships_of_the_line: 6,
                frigates: 2,
                transports: 1,
                morale_q4: 7000,
                admiral: None,
                embarked_corps: Vec::new(),
            },
        );
        scenario.fleets.insert(
            FleetId::from("FLEET_GBR_001"),
            Fleet {
                display_name: "British Fleet".into(),
                owner: PowerId::from("GBR"),
                at_port: None,
                at_sea: Some(SeaZoneId::from("SEA_ENGLISH_CHANNEL")),
                ships_of_the_line: 4,
                frigates: 1,
                transports: 0,
                morale_q4: 7000,
                admiral: None,
                embarked_corps: Vec::new(),
            },
        );

        scenario.diplomacy.insert(
            DiplomaticPairKey::new(PowerId::from("FRA"), PowerId::from("GBR")),
            DiplomaticState::War,
        );

        scenario.coast_links = vec![
            CoastLink {
                area: AreaId::from("AREA_PARIS"),
                sea: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
            },
            CoastLink {
                area: AreaId::from("AREA_LONDON"),
                sea: SeaZoneId::from("SEA_NORTH_SEA"),
            },
            CoastLink {
                area: AreaId::from("AREA_BREST"),
                sea: SeaZoneId::from("SEA_NORTH_SEA"),
            },
        ];
        scenario.sea_adjacency = vec![
            SeaAdjacency {
                from: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
                to: SeaZoneId::from("SEA_NORTH_SEA"),
            },
            SeaAdjacency {
                from: SeaZoneId::from("SEA_NORTH_SEA"),
                to: SeaZoneId::from("SEA_IRISH_SEA"),
            },
        ];

        scenario
    }

    fn placeholder_naval_tables() -> NavalCombatTable {
        let mut results = BTreeMap::new();
        for bucket in ["1:2", "1:1", "2:1"] {
            results.insert(bucket.to_string(), vec![Maybe::placeholder(); 6]);
        }
        NavalCombatTable {
            schema_version: 1,
            ratio_buckets: vec!["1:2".into(), "1:1".into(), "2:1".into()],
            die_faces: 6,
            results,
        }
    }

    fn value_naval_tables() -> NavalCombatTable {
        let mut results = BTreeMap::new();
        for bucket in ["1:2", "1:1", "2:1"] {
            results.insert(
                bucket.to_string(),
                vec![
                    Maybe::Value(NavalResult {
                        attacker_ship_loss: 1,
                        defender_ship_loss: 2,
                        disengage: false,
                    });
                    6
                ],
            );
        }
        NavalCombatTable {
            schema_version: 1,
            ratio_buckets: vec!["1:2".into(), "1:1".into(), "2:1".into()],
            die_faces: 6,
            results,
        }
    }

    #[test]
    fn sea_graph_adjacency() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        assert_eq!(
            graph.adjacent_zones(&SeaZoneId::from("SEA_ENGLISH_CHANNEL")),
            vec![SeaZoneId::from("SEA_NORTH_SEA")]
        );
    }

    #[test]
    fn sea_graph_shortest_path() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        assert_eq!(
            graph.shortest_path_hops(
                &SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
                &SeaZoneId::from("SEA_IRISH_SEA")
            ),
            Some(2)
        );
    }

    #[test]
    fn sea_graph_no_path() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        assert_eq!(
            graph.shortest_path_hops(
                &SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
                &SeaZoneId::from("SEA_BALTIC")
            ),
            None
        );
    }

    #[test]
    fn sea_graph_multiple_zones() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        assert_eq!(
            graph
                .adjacent_zones(&SeaZoneId::from("SEA_NORTH_SEA"))
                .len(),
            2
        );
    }

    #[test]
    fn sea_graph_same_zone_zero_hops() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        assert_eq!(
            graph.shortest_path_hops(
                &SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
                &SeaZoneId::from("SEA_ENGLISH_CHANNEL")
            ),
            Some(0)
        );
    }

    #[test]
    fn validate_fleet_move_ok() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        assert!(validate_fleet_move(&scenario, &graph, &order).is_ok());
    }

    #[test]
    fn validate_fleet_move_not_owner() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("GBR"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        assert!(validate_fleet_move(&scenario, &graph, &order).is_err());
    }

    #[test]
    fn validate_fleet_move_no_path() {
        let scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_BALTIC"),
        };
        assert!(validate_fleet_move(&scenario, &graph, &order).is_err());
    }

    #[test]
    fn validate_fleet_move_requires_at_sea() {
        let mut scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let fleet = scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap();
        fleet.at_port = Some(AreaId::from("AREA_PARIS"));
        fleet.at_sea = None;
        let order = MoveFleetOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        assert!(validate_fleet_move(&scenario, &graph, &order).is_err());
    }

    #[test]
    fn resolve_fleet_move_updates_zone() {
        let mut scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        let _ = resolve_fleet_move(&mut scenario, &graph, &order);
        assert_eq!(
            scenario.fleets[&FleetId::from("FLEET_FRA_001")].at_sea,
            Some(SeaZoneId::from("SEA_NORTH_SEA"))
        );
    }

    #[test]
    fn resolve_fleet_move_emits_event() {
        let mut scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        let events = resolve_fleet_move(&mut scenario, &graph, &order);
        assert!(matches!(
            events.first(),
            Some(Event::FleetMoved { fleet, from, to })
            if fleet == &FleetId::from("FLEET_FRA_001")
                && from == &SeaZoneId::from("SEA_ENGLISH_CHANNEL")
                && to == &SeaZoneId::from("SEA_NORTH_SEA")
        ));
    }

    #[test]
    fn resolve_fleet_move_blockade() {
        let mut scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        let events = resolve_fleet_move(&mut scenario, &graph, &order);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::BlockadeEstablished { fleet, sea_zone }
                if fleet == &FleetId::from("FLEET_FRA_001")
                    && sea_zone == &SeaZoneId::from("SEA_NORTH_SEA")
        )));
        assert!(scenario.areas[&AreaId::from("AREA_LONDON")].blockaded);
    }

    #[test]
    fn blockade_not_emitted_when_only_friendly_port_adjacent() {
        let mut scenario = naval_scenario();
        scenario
            .coast_links
            .retain(|link| link.area != AreaId::from("AREA_LONDON"));
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        let events = resolve_fleet_move(&mut scenario, &graph, &order);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::BlockadeEstablished { .. })));
    }

    #[test]
    fn blockade_emitted_when_enemy_port_adjacent() {
        let mut scenario = naval_scenario();
        let graph = SeaGraph::new(&scenario);
        let order = MoveFleetOrder {
            submitter: PowerId::from("GBR"),
            fleet: FleetId::from("FLEET_GBR_001"),
            destination: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        let events = resolve_fleet_move(&mut scenario, &graph, &order);
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::BlockadeEstablished { .. })));
        assert!(scenario.areas[&AreaId::from("AREA_BREST")].blockaded);
    }

    #[test]
    fn naval_battle_placeholder_rejection() {
        let mut scenario = naval_scenario();
        let order = NavalAttackOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_zone: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
        };
        let events = resolve_naval_battle(&mut scenario, &placeholder_naval_tables(), 0, &order);
        assert!(matches!(
            &events[0],
            Event::OrderRejected(rejection) if rejection.reason_code == "NAVAL_TABLE_PLACEHOLDER"
        ));
    }

    #[test]
    fn naval_battle_value_resolves() {
        let mut scenario = naval_scenario();
        let order = NavalAttackOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_zone: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
        };
        let events = resolve_naval_battle(&mut scenario, &value_naval_tables(), 0, &order);
        assert!(matches!(events[0], Event::NavalBattleResolved { .. }));
    }

    #[test]
    fn ships_lost_applied() {
        let mut scenario = naval_scenario();
        let before = scenario.fleets[&FleetId::from("FLEET_GBR_001")].ships_of_the_line;
        let order = NavalAttackOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_zone: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
        };
        let _ = resolve_naval_battle(&mut scenario, &value_naval_tables(), 0, &order);
        assert!(scenario.fleets[&FleetId::from("FLEET_GBR_001")].ships_of_the_line < before);
    }

    #[test]
    fn ratio_2_1() {
        assert_eq!(naval_ratio_bucket(8, 4), "2:1");
    }

    #[test]
    fn ratio_1_1() {
        assert_eq!(naval_ratio_bucket(5, 4), "1:1");
    }

    #[test]
    fn ratio_1_2() {
        assert_eq!(naval_ratio_bucket(3, 5), "1:2");
    }

    #[test]
    fn deterministic_seed() {
        let mut scenario_a = naval_scenario();
        let mut scenario_b = naval_scenario();
        let order = NavalAttackOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_zone: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
        };
        let events_a = resolve_naval_battle(&mut scenario_a, &value_naval_tables(), 3, &order);
        let events_b = resolve_naval_battle(&mut scenario_b, &value_naval_tables(), 3, &order);
        assert_eq!(events_a, events_b);
    }

    #[test]
    fn naval_battle_requires_enemy_present() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_GBR_001"))
            .unwrap()
            .at_sea = Some(SeaZoneId::from("SEA_NORTH_SEA"));
        let order = NavalAttackOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_zone: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
        };
        let events = resolve_naval_battle(&mut scenario, &value_naval_tables(), 0, &order);
        assert!(matches!(
            &events[0],
            Event::OrderRejected(rejection) if rejection.reason_code == "NO_NAVAL_DEFENDER"
        ));
    }

    #[test]
    fn naval_battle_requires_attacker_in_target_zone() {
        let mut scenario = naval_scenario();
        let order = NavalAttackOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_zone: SeaZoneId::from("SEA_NORTH_SEA"),
        };
        let events = resolve_naval_battle(&mut scenario, &value_naval_tables(), 0, &order);
        assert!(matches!(
            &events[0],
            Event::OrderRejected(rejection) if rejection.reason_code == "FLEET_NOT_IN_TARGET_ZONE"
        ));
    }

    #[test]
    fn naval_battle_defender_sunk_outcome() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_GBR_001"))
            .unwrap()
            .ships_of_the_line = 1;
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_GBR_001"))
            .unwrap()
            .frigates = 0;
        let mut tables = value_naval_tables();
        tables.results.get_mut("2:1").unwrap()[0] = Maybe::Value(NavalResult {
            attacker_ship_loss: 0,
            defender_ship_loss: 4,
            disengage: false,
        });
        let order = NavalAttackOrder {
            submitter: PowerId::from("FRA"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_zone: SeaZoneId::from("SEA_ENGLISH_CHANNEL"),
        };
        let events = resolve_naval_battle(&mut scenario, &tables, 0, &order);
        assert!(matches!(
            &events[0],
            Event::NavalBattleResolved { outcome, .. } if *outcome == NavalOutcome::DefenderSunk
        ));
    }

    #[test]
    fn validate_embark_ok() {
        let scenario = naval_scenario();
        let order = EmbarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
        };
        assert!(validate_embark(&scenario, &order).is_ok());
    }

    #[test]
    fn validate_embark_no_port() {
        let mut scenario = naval_scenario();
        scenario
            .corps
            .get_mut(&CorpsId::from("CORPS_FRA_001"))
            .unwrap()
            .area = AreaId::from("AREA_INLAND");
        let order = EmbarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
        };
        assert!(validate_embark(&scenario, &order).is_err());
    }

    #[test]
    fn validate_embark_fleet_not_in_zone() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .at_sea = Some(SeaZoneId::from("SEA_NORTH_SEA"));
        let order = EmbarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
        };
        assert!(validate_embark(&scenario, &order).is_err());
    }

    #[test]
    fn resolve_embark_marks_corps() {
        let mut scenario = naval_scenario();
        let order = EmbarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
        };
        let _ = resolve_embark(&mut scenario, &order);
        assert!(scenario.fleets[&FleetId::from("FLEET_FRA_001")]
            .embarked_corps
            .contains(&CorpsId::from("CORPS_FRA_001")));
    }

    #[test]
    fn resolve_embark_event() {
        let mut scenario = naval_scenario();
        let order = EmbarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
        };
        let event = resolve_embark(&mut scenario, &order);
        assert!(matches!(
            event,
            Event::CorpsEmbarked { corps, fleet, area }
                if corps == CorpsId::from("CORPS_FRA_001")
                    && fleet == FleetId::from("FLEET_FRA_001")
                    && area == AreaId::from("AREA_PARIS")
        ));
    }

    #[test]
    fn embarked_corps_sorted_deterministically() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .embarked_corps = vec![CorpsId::from("CORPS_ZZZ")];
        scenario.corps.insert(
            CorpsId::from("CORPS_AAA"),
            Corps {
                display_name: "AAA".into(),
                owner: PowerId::from("FRA"),
                area: AreaId::from("AREA_PARIS"),
                infantry_sp: 1,
                cavalry_sp: 0,
                artillery_sp: 0,
                morale_q4: 5000,
                supplied: true,
                leader: None,
            },
        );
        let order = EmbarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_AAA"),
            fleet: FleetId::from("FLEET_FRA_001"),
        };
        let _ = resolve_embark(&mut scenario, &order);
        assert_eq!(
            scenario.fleets[&FleetId::from("FLEET_FRA_001")].embarked_corps,
            vec![CorpsId::from("CORPS_AAA"), CorpsId::from("CORPS_ZZZ")]
        );
    }

    #[test]
    fn validate_disembark_ok() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .at_sea = Some(SeaZoneId::from("SEA_NORTH_SEA"));
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .embarked_corps = vec![CorpsId::from("CORPS_FRA_001")];
        let order = DisembarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_area: AreaId::from("AREA_BREST"),
        };
        assert!(validate_disembark(&scenario, &order).is_ok());
    }

    #[test]
    fn validate_disembark_not_embarked() {
        let scenario = naval_scenario();
        let order = DisembarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_area: AreaId::from("AREA_PARIS"),
        };
        assert!(validate_disembark(&scenario, &order).is_err());
    }

    #[test]
    fn validate_disembark_requires_port() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .embarked_corps = vec![CorpsId::from("CORPS_FRA_001")];
        let order = DisembarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_area: AreaId::from("AREA_INLAND"),
        };
        assert!(validate_disembark(&scenario, &order).is_err());
    }

    #[test]
    fn resolve_disembark_moves_corps() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .at_sea = Some(SeaZoneId::from("SEA_NORTH_SEA"));
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .embarked_corps = vec![CorpsId::from("CORPS_FRA_001")];
        let order = DisembarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_area: AreaId::from("AREA_BREST"),
        };
        let _ = resolve_disembark(&mut scenario, &order);
        assert_eq!(
            scenario.corps[&CorpsId::from("CORPS_FRA_001")].area,
            AreaId::from("AREA_BREST")
        );
    }

    #[test]
    fn resolve_disembark_event() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .at_sea = Some(SeaZoneId::from("SEA_NORTH_SEA"));
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .embarked_corps = vec![CorpsId::from("CORPS_FRA_001")];
        let order = DisembarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_area: AreaId::from("AREA_BREST"),
        };
        let event = resolve_disembark(&mut scenario, &order);
        assert!(matches!(
            event,
            Event::CorpsDisembarked { corps, fleet, area }
                if corps == CorpsId::from("CORPS_FRA_001")
                    && fleet == FleetId::from("FLEET_FRA_001")
                    && area == AreaId::from("AREA_BREST")
        ));
    }

    #[test]
    fn disembark_removes_embarked_marker() {
        let mut scenario = naval_scenario();
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .at_sea = Some(SeaZoneId::from("SEA_NORTH_SEA"));
        scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap()
            .embarked_corps = vec![CorpsId::from("CORPS_FRA_001")];
        let order = DisembarkOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            fleet: FleetId::from("FLEET_FRA_001"),
            target_area: AreaId::from("AREA_BREST"),
        };
        let _ = resolve_disembark(&mut scenario, &order);
        assert!(scenario.fleets[&FleetId::from("FLEET_FRA_001")]
            .embarked_corps
            .is_empty());
    }

    #[test]
    fn fleet_at_port_vs_sea_zone() {
        let mut scenario = naval_scenario();
        let fleet = scenario
            .fleets
            .get_mut(&FleetId::from("FLEET_FRA_001"))
            .unwrap();
        fleet.at_port = Some(AreaId::from("AREA_PARIS"));
        fleet.at_sea = None;
        assert_eq!(fleet.at_port, Some(AreaId::from("AREA_PARIS")));
        assert_eq!(fleet.at_sea, None);
    }
}
