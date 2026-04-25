//! Minor-country state handling (PROMPT.md Phase 8).
//!
//! This module intentionally keeps the first implementation simple and
//! deterministic: the authored `Scenario` still stores a compact
//! `MinorSetup`, while this module exposes a richer runtime-facing state
//! machine and a deterministic activation helper.

use gc1805_core_schema::events::Event;
use gc1805_core_schema::ids::{MinorId, PowerId};
use gc1805_core_schema::scenario::{MinorRelationship, Scenario};
use gc1805_core_schema::tables::{Maybe, MinorActivationRow, MinorActivationTable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinorStatus {
    Independent,
    AlliedFree { patron: PowerId },
    Feudal { patron: PowerId },
    Conquered { by: PowerId },
    InRevolt,
}

pub fn activate_minor(
    scenario: &mut Scenario,
    minor_id: &MinorId,
    tables: &MinorActivationTable,
    rng_seed: u64,
) -> Vec<Event> {
    let Some(current_setup) = scenario.minors.get(minor_id).cloned() else {
        return Vec::new();
    };

    let current_status = status_from_setup(
        current_setup.initial_relationship,
        current_setup.patron.clone(),
    );

    let roll = (rng_seed % 6) as i32;
    let next_status = match tables.rows.get(minor_id.as_str()) {
        Some(Maybe::Value(row)) => pick_status_from_row(row, roll, &current_status),
        _ => placeholder_status(scenario, &current_status, roll),
    };

    if let Some(minor) = scenario.minors.get_mut(minor_id) {
        apply_status_to_setup(minor, &next_status);
    }

    vec![Event::MinorActivated {
        minor: minor_id.clone(),
        new_status: status_name(&next_status).to_owned(),
        patron: patron_for_status(&next_status).cloned(),
    }]
}

pub fn validate_minor_control(
    scenario: &Scenario,
    power: &PowerId,
    minor: &MinorId,
) -> Result<(), String> {
    let setup = scenario
        .minors
        .get(minor)
        .ok_or_else(|| format!("unknown minor `{minor}`"))?;

    let status = status_from_setup(setup.initial_relationship, setup.patron.clone());
    match status {
        MinorStatus::Independent => Err(format!("minor `{minor}` is independent")),
        MinorStatus::InRevolt => Err(format!("minor `{minor}` is in revolt")),
        MinorStatus::AlliedFree { patron } | MinorStatus::Feudal { patron } => {
            if &patron == power {
                Ok(())
            } else {
                Err(format!(
                    "power `{power}` lacks diplomatic control of minor `{minor}` (patron: `{patron}`)"
                ))
            }
        }
        MinorStatus::Conquered { by } => {
            if &by == power {
                Ok(())
            } else {
                Err(format!(
                    "power `{power}` lacks military control of minor `{minor}` (conqueror: `{by}`)"
                ))
            }
        }
    }
}

fn status_from_setup(relationship: MinorRelationship, patron: Option<PowerId>) -> MinorStatus {
    match relationship {
        MinorRelationship::IndependentFree => MinorStatus::Independent,
        MinorRelationship::AlliedFree => MinorStatus::AlliedFree {
            patron: patron.unwrap_or_else(|| PowerId::from("FRA")),
        },
        MinorRelationship::Feudal => MinorStatus::Feudal {
            patron: patron.unwrap_or_else(|| PowerId::from("FRA")),
        },
        MinorRelationship::Conquered => MinorStatus::Conquered {
            by: patron.unwrap_or_else(|| PowerId::from("FRA")),
        },
        MinorRelationship::InRevolt => MinorStatus::InRevolt,
    }
}

fn apply_status_to_setup(
    minor: &mut gc1805_core_schema::scenario::MinorSetup,
    status: &MinorStatus,
) {
    match status {
        MinorStatus::Independent => {
            minor.initial_relationship = MinorRelationship::IndependentFree;
            minor.patron = None;
        }
        MinorStatus::AlliedFree { patron } => {
            minor.initial_relationship = MinorRelationship::AlliedFree;
            minor.patron = Some(patron.clone());
        }
        MinorStatus::Feudal { patron } => {
            minor.initial_relationship = MinorRelationship::Feudal;
            minor.patron = Some(patron.clone());
        }
        MinorStatus::Conquered { by } => {
            minor.initial_relationship = MinorRelationship::Conquered;
            minor.patron = Some(by.clone());
        }
        MinorStatus::InRevolt => {
            minor.initial_relationship = MinorRelationship::InRevolt;
            minor.patron = None;
        }
    }
}

fn patron_for_status(status: &MinorStatus) -> Option<&PowerId> {
    match status {
        MinorStatus::Independent | MinorStatus::InRevolt => None,
        MinorStatus::AlliedFree { patron } | MinorStatus::Feudal { patron } => Some(patron),
        MinorStatus::Conquered { by } => Some(by),
    }
}

fn status_name(status: &MinorStatus) -> &'static str {
    match status {
        MinorStatus::Independent => "Independent",
        MinorStatus::AlliedFree { .. } => "AlliedFree",
        MinorStatus::Feudal { .. } => "Feudal",
        MinorStatus::Conquered { .. } => "Conquered",
        MinorStatus::InRevolt => "InRevolt",
    }
}

fn placeholder_status(scenario: &Scenario, current: &MinorStatus, roll: i32) -> MinorStatus {
    match roll {
        0 | 1 => MinorStatus::Independent,
        2 => MinorStatus::AlliedFree {
            patron: default_patron(scenario, current),
        },
        3 => MinorStatus::Feudal {
            patron: default_patron(scenario, current),
        },
        4 => MinorStatus::Conquered {
            by: default_patron(scenario, current),
        },
        _ => MinorStatus::InRevolt,
    }
}

fn default_patron(scenario: &Scenario, current: &MinorStatus) -> PowerId {
    patron_for_status(current)
        .cloned()
        .or_else(|| scenario.powers.keys().next().cloned())
        .unwrap_or_else(|| PowerId::from("FRA"))
}

fn pick_status_from_row(row: &MinorActivationRow, roll: i32, current: &MinorStatus) -> MinorStatus {
    let die_faces = 6;
    let total_weight: i32 = row.outcomes.values().copied().sum();
    if total_weight <= 0 {
        return current.clone();
    }

    let target = ((roll % die_faces) * total_weight) / die_faces;
    let mut running = 0;
    for (label, weight) in &row.outcomes {
        running += *weight;
        if target < running {
            return parse_outcome_label(label, current);
        }
    }

    row.outcomes
        .keys()
        .next_back()
        .map(|label| parse_outcome_label(label, current))
        .unwrap_or_else(|| current.clone())
}

fn parse_outcome_label(label: &str, current: &MinorStatus) -> MinorStatus {
    let (status, power) = match label.split_once(':') {
        Some((status, power)) => (status, Some(PowerId::from(power))),
        None => (label, None),
    };

    match status {
        "INDEPENDENT" | "INDEPENDENT_FREE" => MinorStatus::Independent,
        "ALLIED_FREE" => MinorStatus::AlliedFree {
            patron: power
                .or_else(|| patron_for_status(current).cloned())
                .unwrap_or_else(|| PowerId::from("FRA")),
        },
        "FEUDAL" => MinorStatus::Feudal {
            patron: power
                .or_else(|| patron_for_status(current).cloned())
                .unwrap_or_else(|| PowerId::from("FRA")),
        },
        "CONQUERED" => MinorStatus::Conquered {
            by: power
                .or_else(|| patron_for_status(current).cloned())
                .unwrap_or_else(|| PowerId::from("FRA")),
        },
        "IN_REVOLT" | "INREVOLT" | "IN_REVOLT_FREE" => MinorStatus::InRevolt,
        _ => current.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::events::Event;
    use gc1805_core_schema::ids::{AreaId, LeaderId};
    use gc1805_core_schema::scenario::{
        Area, Features, GameDate, MinorSetup, MovementRules, Owner, PowerSetup, PowerSlot,
        PowerState, Scenario, TaxPolicy, Terrain, SCHEMA_VERSION,
    };
    use gc1805_core_schema::tables::PlaceholderMarker;
    use serde_json::Value;
    use std::collections::BTreeMap;

    fn scenario_with_minor(
        relationship: MinorRelationship,
        patron: Option<&str>,
        minor_id: &str,
    ) -> Scenario {
        let mut powers = BTreeMap::new();
        powers.insert(
            PowerId::from("FRA"),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: LeaderId::from("LEADER_NAPOLEON"),
                capital: AreaId::from("AREA_PARIS"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 8,
                mobilization_areas: vec![AreaId::from("AREA_PARIS")],
                color_hex: "#000000".into(),
            },
        );
        powers.insert(
            PowerId::from("GBR"),
            PowerSetup {
                display_name: "Great Britain".into(),
                house: "Hanover".into(),
                ruler: LeaderId::from("LEADER_GEORGE_III"),
                capital: AreaId::from("AREA_LONDON"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 4,
                max_depots: 4,
                mobilization_areas: vec![AreaId::from("AREA_LONDON")],
                color_hex: "#ffffff".into(),
            },
        );

        let mut power_state = BTreeMap::new();
        power_state.insert(
            PowerId::from("FRA"),
            PowerState {
                treasury: 0,
                manpower: 0,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        power_state.insert(
            PowerId::from("GBR"),
            PowerState {
                treasury: 0,
                manpower: 0,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );

        let mut areas = BTreeMap::new();
        areas.insert(
            AreaId::from("AREA_PARIS"),
            area(
                "Paris",
                Owner::Power(PowerSlot {
                    power: PowerId::from("FRA"),
                }),
            ),
        );
        areas.insert(
            AreaId::from("AREA_LONDON"),
            area(
                "London",
                Owner::Power(PowerSlot {
                    power: PowerId::from("GBR"),
                }),
            ),
        );
        areas.insert(
            AreaId::from("AREA_TEST"),
            area(
                "Test",
                Owner::Minor(gc1805_core_schema::scenario::MinorSlot {
                    minor: MinorId::from(minor_id),
                }),
            ),
        );

        let mut minors = BTreeMap::new();
        minors.insert(
            MinorId::from(minor_id),
            MinorSetup {
                display_name: "Test Minor".into(),
                home_areas: vec![AreaId::from("AREA_TEST")],
                initial_relationship: relationship,
                patron: patron.map(PowerId::from),
                starting_force_level: 1,
            },
        );

        Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "test".into(),
            name: "Test".into(),
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
            minors,
            leaders: BTreeMap::new(),
            areas,
            sea_zones: BTreeMap::new(),
            corps: BTreeMap::new(),
            fleets: BTreeMap::new(),
            diplomacy: BTreeMap::new(),
            adjacency: Vec::new(),
            coast_links: Vec::new(),
            sea_adjacency: Vec::new(),
        }
    }

    fn area(name: &str, owner: Owner) -> Area {
        Area {
            display_name: name.into(),
            owner,
            terrain: Terrain::Open,
            fort_level: 0,
            money_yield: Maybe::Placeholder(PlaceholderMarker::new()),
            manpower_yield: Maybe::Placeholder(PlaceholderMarker::new()),
            capital_of: None,
            port: false,
            blockaded: false,
            map_x: 0,
            map_y: 0,
        }
    }

    fn placeholder_table() -> MinorActivationTable {
        MinorActivationTable {
            schema_version: 1,
            rows: BTreeMap::new(),
        }
    }

    fn value_table(minor_id: &str, outcomes: &[(&str, i32)]) -> MinorActivationTable {
        let mut rows = BTreeMap::new();
        let mut map = BTreeMap::new();
        for (label, weight) in outcomes {
            map.insert((*label).to_owned(), *weight);
        }
        rows.insert(
            minor_id.to_owned(),
            Maybe::Value(MinorActivationRow {
                trigger: "DEFAULT".into(),
                outcomes: map,
            }),
        );
        MinorActivationTable {
            schema_version: 1,
            rows,
        }
    }

    #[test]
    fn status_from_setup_independent() {
        assert_eq!(
            status_from_setup(MinorRelationship::IndependentFree, None),
            MinorStatus::Independent
        );
    }

    #[test]
    fn status_from_setup_allied() {
        assert_eq!(
            status_from_setup(MinorRelationship::AlliedFree, Some(PowerId::from("GBR"))),
            MinorStatus::AlliedFree {
                patron: PowerId::from("GBR")
            }
        );
    }

    #[test]
    fn status_from_setup_feudal() {
        assert_eq!(
            status_from_setup(MinorRelationship::Feudal, Some(PowerId::from("FRA"))),
            MinorStatus::Feudal {
                patron: PowerId::from("FRA")
            }
        );
    }

    #[test]
    fn status_from_setup_conquered() {
        assert_eq!(
            status_from_setup(MinorRelationship::Conquered, Some(PowerId::from("FRA"))),
            MinorStatus::Conquered {
                by: PowerId::from("FRA")
            }
        );
    }

    #[test]
    fn status_from_setup_revolt() {
        assert_eq!(
            status_from_setup(MinorRelationship::InRevolt, None),
            MinorStatus::InRevolt
        );
    }

    #[test]
    fn validate_control_allied_patron_ok() {
        let scenario = scenario_with_minor(MinorRelationship::AlliedFree, Some("GBR"), "MINOR_X");
        assert!(validate_minor_control(
            &scenario,
            &PowerId::from("GBR"),
            &MinorId::from("MINOR_X")
        )
        .is_ok());
    }

    #[test]
    fn validate_control_allied_wrong_power_fails() {
        let scenario = scenario_with_minor(MinorRelationship::AlliedFree, Some("GBR"), "MINOR_X");
        assert!(validate_minor_control(
            &scenario,
            &PowerId::from("FRA"),
            &MinorId::from("MINOR_X")
        )
        .is_err());
    }

    #[test]
    fn validate_control_feudal_patron_ok() {
        let scenario = scenario_with_minor(MinorRelationship::Feudal, Some("FRA"), "MINOR_X");
        assert!(validate_minor_control(
            &scenario,
            &PowerId::from("FRA"),
            &MinorId::from("MINOR_X")
        )
        .is_ok());
    }

    #[test]
    fn validate_control_conqueror_ok() {
        let scenario = scenario_with_minor(MinorRelationship::Conquered, Some("FRA"), "MINOR_X");
        assert!(validate_minor_control(
            &scenario,
            &PowerId::from("FRA"),
            &MinorId::from("MINOR_X")
        )
        .is_ok());
    }

    #[test]
    fn validate_control_independent_fails() {
        let scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        assert!(validate_minor_control(
            &scenario,
            &PowerId::from("FRA"),
            &MinorId::from("MINOR_X")
        )
        .is_err());
    }

    #[test]
    fn validate_control_missing_minor_fails() {
        let scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        assert!(validate_minor_control(
            &scenario,
            &PowerId::from("FRA"),
            &MinorId::from("MINOR_Y")
        )
        .is_err());
    }

    #[test]
    fn placeholder_activation_roll_zero_stays_independent() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        let events = activate_minor(
            &mut scenario,
            &MinorId::from("MINOR_X"),
            &placeholder_table(),
            0,
        );
        assert_eq!(
            scenario.minors[&MinorId::from("MINOR_X")].initial_relationship,
            MinorRelationship::IndependentFree
        );
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn placeholder_activation_roll_two_becomes_allied() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        activate_minor(
            &mut scenario,
            &MinorId::from("MINOR_X"),
            &placeholder_table(),
            2,
        );
        let minor = &scenario.minors[&MinorId::from("MINOR_X")];
        assert_eq!(minor.initial_relationship, MinorRelationship::AlliedFree);
        assert_eq!(minor.patron.as_ref(), Some(&PowerId::from("FRA")));
    }

    #[test]
    fn placeholder_activation_roll_three_becomes_feudal() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        activate_minor(
            &mut scenario,
            &MinorId::from("MINOR_X"),
            &placeholder_table(),
            3,
        );
        assert_eq!(
            scenario.minors[&MinorId::from("MINOR_X")].initial_relationship,
            MinorRelationship::Feudal
        );
    }

    #[test]
    fn placeholder_activation_roll_four_becomes_conquered() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        activate_minor(
            &mut scenario,
            &MinorId::from("MINOR_X"),
            &placeholder_table(),
            4,
        );
        assert_eq!(
            scenario.minors[&MinorId::from("MINOR_X")].initial_relationship,
            MinorRelationship::Conquered
        );
    }

    #[test]
    fn placeholder_activation_roll_five_becomes_revolt() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        activate_minor(
            &mut scenario,
            &MinorId::from("MINOR_X"),
            &placeholder_table(),
            5,
        );
        assert_eq!(
            scenario.minors[&MinorId::from("MINOR_X")].initial_relationship,
            MinorRelationship::InRevolt
        );
    }

    #[test]
    fn placeholder_activation_is_deterministic() {
        let mut s1 = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        let mut s2 = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        let e1 = activate_minor(&mut s1, &MinorId::from("MINOR_X"), &placeholder_table(), 4);
        let e2 = activate_minor(&mut s2, &MinorId::from("MINOR_X"), &placeholder_table(), 4);
        assert_eq!(e1, e2);
        assert_eq!(
            s1.minors[&MinorId::from("MINOR_X")].initial_relationship,
            s2.minors[&MinorId::from("MINOR_X")].initial_relationship
        );
    }

    #[test]
    fn value_table_single_outcome_applies() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        let table = value_table("MINOR_X", &[("ALLIED_FREE:GBR", 4096)]);
        activate_minor(&mut scenario, &MinorId::from("MINOR_X"), &table, 1);
        let minor = &scenario.minors[&MinorId::from("MINOR_X")];
        assert_eq!(minor.initial_relationship, MinorRelationship::AlliedFree);
        assert_eq!(minor.patron.as_ref(), Some(&PowerId::from("GBR")));
    }

    #[test]
    fn value_table_weighted_pick_can_choose_last_bucket() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        let table = value_table(
            "MINOR_X",
            &[("ALLIED_FREE:FRA", 1024), ("FEUDAL:FRA", 3072)],
        );
        activate_minor(&mut scenario, &MinorId::from("MINOR_X"), &table, 5);
        assert_eq!(
            scenario.minors[&MinorId::from("MINOR_X")].initial_relationship,
            MinorRelationship::Feudal
        );
    }

    #[test]
    fn value_table_unknown_label_keeps_current_status() {
        let mut scenario = scenario_with_minor(MinorRelationship::Feudal, Some("FRA"), "MINOR_X");
        let table = value_table("MINOR_X", &[("MYSTERY", 4096)]);
        activate_minor(&mut scenario, &MinorId::from("MINOR_X"), &table, 0);
        assert_eq!(
            scenario.minors[&MinorId::from("MINOR_X")].initial_relationship,
            MinorRelationship::Feudal
        );
    }

    #[test]
    fn activation_event_contains_status_and_patron() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        let table = value_table("MINOR_X", &[("ALLIED_FREE:GBR", 4096)]);
        let events = activate_minor(&mut scenario, &MinorId::from("MINOR_X"), &table, 0);
        match &events[0] {
            Event::MinorActivated {
                minor,
                new_status,
                patron,
            } => {
                assert_eq!(minor, &MinorId::from("MINOR_X"));
                assert_eq!(new_status, "AlliedFree");
                assert_eq!(patron.as_ref(), Some(&PowerId::from("GBR")));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn activate_unknown_minor_returns_no_events() {
        let mut scenario = scenario_with_minor(MinorRelationship::IndependentFree, None, "MINOR_X");
        let events = activate_minor(
            &mut scenario,
            &MinorId::from("MINOR_Z"),
            &placeholder_table(),
            0,
        );
        assert!(events.is_empty());
    }

    #[test]
    fn parse_outcome_without_patron_reuses_current_patron() {
        let current = MinorStatus::Feudal {
            patron: PowerId::from("GBR"),
        };
        assert_eq!(
            parse_outcome_label("CONQUERED", &current),
            MinorStatus::Conquered {
                by: PowerId::from("GBR")
            }
        );
    }

    #[test]
    fn default_patron_prefers_current_patron() {
        let scenario = scenario_with_minor(MinorRelationship::Feudal, Some("GBR"), "MINOR_X");
        let current = MinorStatus::Feudal {
            patron: PowerId::from("GBR"),
        };
        assert_eq!(default_patron(&scenario, &current), PowerId::from("GBR"));
    }

    #[test]
    fn iberian_guerilla_flag_present_in_minors_table() {
        let raw = include_str!("../../../data/tables/minors.json");
        let value: Value = serde_json::from_str(raw).unwrap();
        let defs = value["minor_definitions"].as_array().unwrap();
        let portugal = defs.iter().find(|m| m["id"] == "MINOR_PORTUGAL").unwrap();
        let rules = portugal["special_rules"].as_array().unwrap();
        assert!(rules.iter().any(|r| r == "GUERILLA_PRONE"));
    }
}
