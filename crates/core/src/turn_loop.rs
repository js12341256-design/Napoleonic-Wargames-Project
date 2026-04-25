//! Full deterministic turn orchestration.
//!
//! Phase order:
//! 1. Economic
//! 2. Movement
//! 3. Combat
//! 4. Supply (stub)
//! 5. Political (stub)

use gc1805_core_schema::{
    canonical::canonical_hash,
    events::{Event, OrderRejected},
    scenario::Scenario,
    tables::{AttritionTable, CombatTable, EconomyTable, MoraleTable},
};

use crate::{
    combat::{resolve_battle, validate_attack},
    economy::{apply_economic_order, resolve_economic_phase, validate_economic_order},
    movement::{resolve_order, validate_or_reject},
    orders::{AttackOrder, Order},
};

#[derive(Debug, Clone)]
pub struct AllTables {
    pub economy: EconomyTable,
    pub combat: CombatTable,
    pub morale: MoraleTable,
    pub attrition: AttritionTable,
}

#[derive(Debug, Clone, Default)]
pub struct TurnInput {
    pub economic_orders: Vec<Order>,
    pub movement_orders: Vec<Order>,
    pub attack_orders: Vec<AttackOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnOutput {
    pub events: Vec<Event>,
    pub state_hash: String,
}

pub fn run_turn(
    scenario: &mut Scenario,
    tables: &AllTables,
    input: TurnInput,
    rng_seed: u64,
) -> TurnOutput {
    let _serialized = serde_json::to_string(scenario)
        .expect("scenario must serialize to JSON before turn hashing");

    let original_turn = scenario.current_turn;
    let mut events = vec![Event::TurnStarted {
        turn: original_turn,
    }];

    for order in &input.economic_orders {
        match validate_economic_order(scenario, &tables.economy, order) {
            Ok(()) => events.push(apply_economic_order(scenario, &tables.economy, order)),
            Err(message) => events.push(Event::OrderRejected(OrderRejected {
                reason_code: "ECONOMIC_ORDER_REJECTED".into(),
                message,
            })),
        }
    }
    events.extend(resolve_economic_phase(scenario, &tables.economy));
    events.push(Event::PhaseCompleted {
        turn: original_turn,
        phase_name: "ECONOMIC".into(),
    });

    for order in &input.movement_orders {
        match validate_or_reject(scenario, order) {
            Ok(plan) => events.push(resolve_order(scenario, order, plan)),
            Err(rejection) => events.push(rejection),
        }
    }
    events.push(Event::PhaseCompleted {
        turn: original_turn,
        phase_name: "MOVEMENT".into(),
    });

    for (index, order) in input.attack_orders.iter().enumerate() {
        match validate_attack(scenario, order) {
            Ok(()) => events.extend(resolve_battle(
                scenario,
                &tables.combat,
                &tables.morale,
                rng_seed.wrapping_add(index as u64),
                order,
            )),
            Err(message) => events.push(Event::OrderRejected(OrderRejected {
                reason_code: "ATTACK_ORDER_REJECTED".into(),
                message,
            })),
        }
    }
    events.push(Event::PhaseCompleted {
        turn: original_turn,
        phase_name: "COMBAT".into(),
    });
    events.push(Event::PhaseCompleted {
        turn: original_turn,
        phase_name: "SUPPLY".into(),
    });
    events.push(Event::PhaseCompleted {
        turn: original_turn,
        phase_name: "POLITICAL".into(),
    });

    let _ = &tables.attrition;

    scenario.current_turn += 1;

    let _serialized_after =
        serde_json::to_string(scenario).expect("scenario must serialize to JSON after turn");
    let state_hash = canonical_hash(scenario).expect("turn state must canonicalize for hashing");

    events.push(Event::TurnCompleted {
        turn: original_turn,
        state_hash: state_hash.clone(),
    });

    TurnOutput { events, state_hash }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::{
        combat_types::BattleOutcome,
        ids::{AreaId, CorpsId, LeaderId, PowerId},
        scenario::{
            Area, AreaAdjacency, Corps, DiplomaticPairKey, DiplomaticState, Features, GameDate,
            MovementRules, Owner, PowerSetup, PowerSlot, PowerState, Scenario, TaxPolicy, Terrain,
            SCHEMA_VERSION,
        },
        tables::{CombatResult, FormationEntry, Maybe, PlaceholderMarker, TerrainModifier},
    };
    use std::collections::BTreeMap;

    fn fra() -> PowerId {
        PowerId::from("FRA")
    }

    fn aus() -> PowerId {
        PowerId::from("AUS")
    }

    fn area_paris() -> AreaId {
        AreaId::from("AREA_PARIS")
    }

    fn area_vienna() -> AreaId {
        AreaId::from("AREA_VIENNA")
    }

    fn corps_fra() -> CorpsId {
        CorpsId::from("CORPS_FRA_001")
    }

    fn corps_aus() -> CorpsId {
        CorpsId::from("CORPS_AUS_001")
    }

    fn scenario_fixture() -> Scenario {
        let mut powers = BTreeMap::new();
        powers.insert(
            fra(),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: LeaderId::from("LEADER_NAPOLEON"),
                capital: area_paris(),
                starting_treasury: 100,
                starting_manpower: 50,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 3,
                mobilization_areas: vec![area_paris()],
                color_hex: "#2a3a6a".into(),
            },
        );
        powers.insert(
            aus(),
            PowerSetup {
                display_name: "Austria".into(),
                house: "Habsburg".into(),
                ruler: LeaderId::from("LEADER_CHARLES"),
                capital: area_vienna(),
                starting_treasury: 80,
                starting_manpower: 40,
                starting_pp: 0,
                max_corps: 8,
                max_depots: 3,
                mobilization_areas: vec![area_vienna()],
                color_hex: "#c0c0c0".into(),
            },
        );

        let mut areas = BTreeMap::new();
        areas.insert(
            area_paris(),
            Area {
                display_name: "Paris".into(),
                owner: Owner::Power(PowerSlot { power: fra() }),
                terrain: Terrain::Urban,
                fort_level: 2,
                money_yield: Maybe::Value(10),
                manpower_yield: Maybe::Value(0),
                capital_of: Some(fra()),
                port: false,
                blockaded: false,
                map_x: 0,
                map_y: 0,
            },
        );
        areas.insert(
            area_vienna(),
            Area {
                display_name: "Vienna".into(),
                owner: Owner::Power(PowerSlot { power: aus() }),
                terrain: Terrain::Open,
                fort_level: 1,
                money_yield: Maybe::Value(8),
                manpower_yield: Maybe::Value(0),
                capital_of: Some(aus()),
                port: false,
                blockaded: false,
                map_x: 100,
                map_y: 0,
            },
        );

        let mut corps = BTreeMap::new();
        corps.insert(
            corps_fra(),
            Corps {
                display_name: "I Corps".into(),
                owner: fra(),
                area: area_paris(),
                infantry_sp: 4,
                cavalry_sp: 1,
                artillery_sp: 1,
                morale_q4: 9000,
                supplied: true,
                leader: None,
            },
        );
        corps.insert(
            corps_aus(),
            Corps {
                display_name: "AUS I Corps".into(),
                owner: aus(),
                area: area_vienna(),
                infantry_sp: 3,
                cavalry_sp: 1,
                artillery_sp: 0,
                morale_q4: 8000,
                supplied: true,
                leader: None,
            },
        );

        let mut power_state = BTreeMap::new();
        power_state.insert(
            fra(),
            PowerState {
                treasury: 100,
                manpower: 50,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        power_state.insert(
            aus(),
            PowerState {
                treasury: 80,
                manpower: 40,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );

        let mut diplomacy = BTreeMap::new();
        diplomacy.insert(DiplomaticPairKey::new(fra(), aus()), DiplomaticState::War);

        Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "turn-loop".into(),
            name: "Turn Loop Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules {
                max_corps_per_area: Maybe::Value(2),
                movement_hops_per_turn: Maybe::Value(2),
                forced_march_extra_hops: Maybe::Value(1),
                forced_march_morale_loss_q4: Maybe::Value(500),
            },
            current_turn: 0,
            power_state,
            production_queue: vec![],
            replacement_queue: vec![],
            subsidy_queue: vec![],
            powers,
            minors: BTreeMap::new(),
            leaders: BTreeMap::new(),
            areas,
            sea_zones: BTreeMap::new(),
            corps,
            fleets: BTreeMap::new(),
            diplomacy,
            adjacency: vec![
                AreaAdjacency {
                    from: area_paris(),
                    to: area_vienna(),
                    cost: Maybe::Value(1),
                },
                AreaAdjacency {
                    from: area_vienna(),
                    to: area_paris(),
                    cost: Maybe::Value(1),
                },
            ],
            coast_links: vec![],
            sea_adjacency: vec![],
        }
    }

    fn standard_tables() -> AllTables {
        let mut results = BTreeMap::new();
        let combat_result = CombatResult {
            attacker_sp_loss: 1,
            defender_sp_loss: 2,
            attacker_morale_q4: -500,
            defender_morale_q4: -1000,
            retreat_hexes: 1,
        };
        for bucket in ["1:3", "1:2", "1:1", "3:2", "2:1", "3:1"] {
            results.insert(
                bucket.to_string(),
                vec![
                    Maybe::Value(combat_result.clone()),
                    Maybe::Value(combat_result.clone()),
                    Maybe::Value(combat_result.clone()),
                    Maybe::Value(combat_result.clone()),
                    Maybe::Value(combat_result.clone()),
                    Maybe::Value(combat_result.clone()),
                ],
            );
        }

        let mut formation_matrix = BTreeMap::new();
        formation_matrix.insert(
            "LINE_vs_LINE".into(),
            FormationEntry {
                att_col_shift: 0,
                def_col_shift: 0,
            },
        );

        let mut terrain_modifiers = BTreeMap::new();
        terrain_modifiers.insert("OPEN".into(), TerrainModifier { att_col_shift: 0 });
        terrain_modifiers.insert("URBAN".into(), TerrainModifier { att_col_shift: -1 });

        AllTables {
            economy: EconomyTable {
                schema_version: 1,
                corps_maintenance_per_sp: Maybe::Value(2),
                fleet_maintenance_per_ship: Maybe::Value(5),
                tax_policy_multiplier_low_q4: Maybe::Value(8_000),
                tax_policy_multiplier_standard_q4: Maybe::Value(10_000),
                tax_policy_multiplier_heavy_q4: Maybe::Value(12_000),
                corps_build_cost_money: Maybe::Value(50),
                corps_build_cost_manpower: Maybe::Value(10),
                corps_production_lag_turns: Maybe::Value(3),
                corps_minimum_sp: Maybe::Placeholder(Default::default()),
                new_corps_morale_q4: Maybe::Value(8_000),
                fleet_build_cost_money: Maybe::Value(80),
                fleet_production_lag_turns: Maybe::Value(2),
                depot_build_cost: Maybe::Placeholder(Default::default()),
                max_depots_default: Maybe::Placeholder(Default::default()),
                manpower_recovery_q12: Maybe::Placeholder(Default::default()),
                manpower_recovery_lag_turns: Maybe::Placeholder(Default::default()),
            },
            combat: CombatTable {
                schema_version: 1,
                ratio_buckets: vec![
                    "1:3".into(),
                    "1:2".into(),
                    "1:1".into(),
                    "3:2".into(),
                    "2:1".into(),
                    "3:1".into(),
                ],
                die_faces: 6,
                formations: vec!["LINE".into()],
                formation_matrix,
                terrain_modifiers,
                results,
            },
            morale: MoraleTable {
                schema_version: 1,
                retreat_threshold_q4: Maybe::Value(5_000),
                rout_threshold_q4: Maybe::Value(2_000),
                recovery_per_turn_q4: Maybe::Value(200),
            },
            attrition: AttritionTable {
                schema_version: 1,
                rows: BTreeMap::new(),
            },
        }
    }

    fn placeholder_tables() -> AllTables {
        let mut results = BTreeMap::new();
        for bucket in ["1:3", "1:2", "1:1", "3:2", "2:1", "3:1"] {
            results.insert(
                bucket.to_string(),
                vec![
                    Maybe::Placeholder(PlaceholderMarker::new()),
                    Maybe::Placeholder(PlaceholderMarker::new()),
                    Maybe::Placeholder(PlaceholderMarker::new()),
                    Maybe::Placeholder(PlaceholderMarker::new()),
                    Maybe::Placeholder(PlaceholderMarker::new()),
                    Maybe::Placeholder(PlaceholderMarker::new()),
                ],
            );
        }

        AllTables {
            economy: EconomyTable {
                schema_version: 1,
                corps_maintenance_per_sp: Maybe::Placeholder(Default::default()),
                fleet_maintenance_per_ship: Maybe::Placeholder(Default::default()),
                tax_policy_multiplier_low_q4: Maybe::Placeholder(Default::default()),
                tax_policy_multiplier_standard_q4: Maybe::Placeholder(Default::default()),
                tax_policy_multiplier_heavy_q4: Maybe::Placeholder(Default::default()),
                corps_build_cost_money: Maybe::Placeholder(Default::default()),
                corps_build_cost_manpower: Maybe::Placeholder(Default::default()),
                corps_production_lag_turns: Maybe::Placeholder(Default::default()),
                corps_minimum_sp: Maybe::Placeholder(Default::default()),
                new_corps_morale_q4: Maybe::Placeholder(Default::default()),
                fleet_build_cost_money: Maybe::Placeholder(Default::default()),
                fleet_production_lag_turns: Maybe::Placeholder(Default::default()),
                depot_build_cost: Maybe::Placeholder(Default::default()),
                max_depots_default: Maybe::Placeholder(Default::default()),
                manpower_recovery_q12: Maybe::Placeholder(Default::default()),
                manpower_recovery_lag_turns: Maybe::Placeholder(Default::default()),
            },
            combat: CombatTable {
                schema_version: 1,
                ratio_buckets: vec![
                    "1:3".into(),
                    "1:2".into(),
                    "1:1".into(),
                    "3:2".into(),
                    "2:1".into(),
                    "3:1".into(),
                ],
                die_faces: 6,
                formations: vec!["LINE".into()],
                formation_matrix: BTreeMap::new(),
                terrain_modifiers: BTreeMap::new(),
                results,
            },
            morale: MoraleTable {
                schema_version: 1,
                retreat_threshold_q4: Maybe::Placeholder(Default::default()),
                rout_threshold_q4: Maybe::Placeholder(Default::default()),
                recovery_per_turn_q4: Maybe::Placeholder(Default::default()),
            },
            attrition: AttritionTable {
                schema_version: 1,
                rows: BTreeMap::new(),
            },
        }
    }

    fn empty_input() -> TurnInput {
        TurnInput::default()
    }

    #[test]
    fn run_turn_increments_current_turn() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert_eq!(scenario.current_turn, 1);
        assert!(!output.state_hash.is_empty());
    }

    #[test]
    fn run_turn_emits_turn_started() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert!(matches!(
            output.events.first(),
            Some(Event::TurnStarted { turn: 0 })
        ));
    }

    #[test]
    fn run_turn_emits_turn_completed() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert!(output
            .events
            .iter()
            .any(|event| matches!(event, Event::TurnCompleted { turn: 0, .. })));
    }

    #[test]
    fn state_hash_is_64_hex_chars() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert_eq!(output.state_hash.len(), 64);
    }

    #[test]
    fn state_hash_hex_chars_only() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert!(output.state_hash.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn determinism_same_seed_same_hash() {
        let mut scenario_a = scenario_fixture();
        let mut scenario_b = scenario_fixture();
        let tables = standard_tables();
        let output_a = run_turn(&mut scenario_a, &tables, empty_input(), 7);
        let output_b = run_turn(&mut scenario_b, &tables, empty_input(), 7);
        assert_eq!(output_a.state_hash, output_b.state_hash);
        assert_eq!(
            serde_json::to_string(&output_a.events).expect("events serialize"),
            serde_json::to_string(&output_b.events).expect("events serialize")
        );
    }

    #[test]
    fn empty_orders_completes() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert!(!output.events.is_empty());
        assert!(matches!(
            output.events.last(),
            Some(Event::TurnCompleted { .. })
        ));
    }

    #[test]
    fn phase_completed_count_is_5() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        let phase_events: Vec<_> = output
            .events
            .iter()
            .filter(|event| matches!(event, Event::PhaseCompleted { .. }))
            .collect();
        assert_eq!(phase_events.len(), 5);
    }

    #[test]
    fn turn_started_correct_number() {
        let mut scenario = scenario_fixture();
        scenario.current_turn = 9;
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert!(matches!(
            output.events.first(),
            Some(Event::TurnStarted { turn: 9 })
        ));
    }

    #[test]
    fn economic_orders_run() {
        let mut scenario = scenario_fixture();
        let input = TurnInput {
            economic_orders: vec![Order::SetTaxPolicy(crate::orders::SetTaxPolicyOrder {
                submitter: fra(),
                policy: TaxPolicy::Heavy,
            })],
            ..TurnInput::default()
        };
        let output = run_turn(&mut scenario, &standard_tables(), input, 0);
        assert_eq!(scenario.power_state[&fra()].tax_policy, TaxPolicy::Heavy);
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::TaxPolicySet {
                power,
                new_policy: TaxPolicy::Heavy
            } if power == &fra()
        )));
    }

    #[test]
    fn multiple_turns_increment_counter() {
        let mut scenario = scenario_fixture();
        let tables = standard_tables();
        let _ = run_turn(&mut scenario, &tables, empty_input(), 0);
        let _ = run_turn(&mut scenario, &tables, empty_input(), 0);
        assert_eq!(scenario.current_turn, 2);
    }

    #[test]
    fn turn_output_events_not_empty() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert!(!output.events.is_empty());
    }

    #[test]
    fn state_hash_changes_after_turn() {
        let mut scenario = scenario_fixture();
        let tables = standard_tables();
        let output_1 = run_turn(&mut scenario, &tables, empty_input(), 0);
        let output_2 = run_turn(&mut scenario, &tables, empty_input(), 0);
        assert_ne!(output_1.state_hash, output_2.state_hash);
    }

    #[test]
    fn all_placeholder_tables_still_completes() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &placeholder_tables(), empty_input(), 0);
        assert_eq!(scenario.current_turn, 1);
        assert!(matches!(
            output.events.last(),
            Some(Event::TurnCompleted { .. })
        ));
    }

    #[test]
    fn movement_order_runs() {
        let mut scenario = scenario_fixture();
        let input = TurnInput {
            movement_orders: vec![Order::Move(crate::orders::MoveOrder {
                submitter: fra(),
                corps: corps_fra(),
                to: area_vienna(),
            })],
            ..TurnInput::default()
        };
        let output = run_turn(&mut scenario, &standard_tables(), input, 0);
        assert_eq!(scenario.corps[&corps_fra()].area, area_vienna());
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::MovementResolved(m) if m.corps == corps_fra() && m.to == area_vienna()
        )));
    }

    #[test]
    fn combat_order_runs() {
        let mut scenario = scenario_fixture();
        let input = TurnInput {
            attack_orders: vec![AttackOrder {
                submitter: fra(),
                attacking_corps: vec![corps_fra()],
                target_area: area_vienna(),
                formation: "LINE".into(),
            }],
            ..TurnInput::default()
        };
        let output = run_turn(&mut scenario, &standard_tables(), input, 3);
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::BattleResolved {
                area,
                attacker,
                defender,
                ..
            } if area == &area_vienna() && attacker == &fra() && defender == &aus()
        )));
    }

    #[test]
    fn phase_names_are_in_order() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        let phases: Vec<_> = output
            .events
            .iter()
            .filter_map(|event| {
                if let Event::PhaseCompleted { phase_name, .. } = event {
                    Some(phase_name.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            phases,
            vec!["ECONOMIC", "MOVEMENT", "COMBAT", "SUPPLY", "POLITICAL"]
        );
    }

    #[test]
    fn turn_completed_hash_matches_output_hash() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        let event_hash = output.events.iter().find_map(|event| {
            if let Event::TurnCompleted { state_hash, .. } = event {
                Some(state_hash.clone())
            } else {
                None
            }
        });
        assert_eq!(event_hash, Some(output.state_hash));
    }

    #[test]
    fn large_turn_number_supported() {
        let mut scenario = scenario_fixture();
        scenario.current_turn = u32::MAX - 1;
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert_eq!(scenario.current_turn, u32::MAX);
        assert!(matches!(
            output.events.first(),
            Some(Event::TurnStarted { turn }) if *turn == u32::MAX - 1
        ));
    }

    #[test]
    fn two_powers_receive_income_events() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        let income_events = output
            .events
            .iter()
            .filter(|event| matches!(event, Event::IncomePaid { .. }))
            .count();
        assert_eq!(income_events, 2);
    }

    #[test]
    fn attack_with_placeholder_combat_table_rejects_but_completes_turn() {
        let mut scenario = scenario_fixture();
        let input = TurnInput {
            attack_orders: vec![AttackOrder {
                submitter: fra(),
                attacking_corps: vec![corps_fra()],
                target_area: area_vienna(),
                formation: "LINE".into(),
            }],
            ..TurnInput::default()
        };
        let output = run_turn(&mut scenario, &placeholder_tables(), input, 0);
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::OrderRejected(rejection) if rejection.reason_code == "COMBAT_TABLE_PLACEHOLDER"
        )));
        assert!(matches!(
            output.events.last(),
            Some(Event::TurnCompleted { .. })
        ));
    }

    #[test]
    fn invalid_movement_order_emits_rejection() {
        let mut scenario = scenario_fixture();
        let input = TurnInput {
            movement_orders: vec![Order::Move(crate::orders::MoveOrder {
                submitter: fra(),
                corps: corps_fra(),
                to: AreaId::from("AREA_NOWHERE"),
            })],
            ..TurnInput::default()
        };
        let output = run_turn(&mut scenario, &standard_tables(), input, 0);
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::OrderRejected(rejection) if rejection.reason_code == "UNKNOWN_AREA"
        )));
    }

    #[test]
    fn invalid_attack_order_emits_rejection() {
        let mut scenario = scenario_fixture();
        let input = TurnInput {
            attack_orders: vec![AttackOrder {
                submitter: fra(),
                attacking_corps: vec![],
                target_area: area_vienna(),
                formation: "LINE".into(),
            }],
            ..TurnInput::default()
        };
        let output = run_turn(&mut scenario, &standard_tables(), input, 0);
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::OrderRejected(rejection) if rejection.reason_code == "ATTACK_ORDER_REJECTED"
        )));
    }

    #[test]
    fn supply_and_political_stub_events_exist() {
        let mut scenario = scenario_fixture();
        let output = run_turn(&mut scenario, &standard_tables(), empty_input(), 0);
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::PhaseCompleted { phase_name, .. } if phase_name == "SUPPLY"
        )));
        assert!(output.events.iter().any(|event| matches!(
            event,
            Event::PhaseCompleted { phase_name, .. } if phase_name == "POLITICAL"
        )));
    }

    #[test]
    fn combat_outcome_is_deterministic_for_same_seed() {
        let mut scenario_a = scenario_fixture();
        let mut scenario_b = scenario_fixture();
        let tables = standard_tables();
        let input = TurnInput {
            attack_orders: vec![AttackOrder {
                submitter: fra(),
                attacking_corps: vec![corps_fra()],
                target_area: area_vienna(),
                formation: "LINE".into(),
            }],
            ..TurnInput::default()
        };
        let output_a = run_turn(&mut scenario_a, &tables, input.clone(), 11);
        let output_b = run_turn(&mut scenario_b, &tables, input, 11);
        let outcome_a = output_a.events.iter().find_map(|event| {
            if let Event::BattleResolved { outcome, .. } = event {
                Some(outcome.clone())
            } else {
                None
            }
        });
        let outcome_b = output_b.events.iter().find_map(|event| {
            if let Event::BattleResolved { outcome, .. } = event {
                Some(outcome.clone())
            } else {
                None
            }
        });
        assert_eq!(outcome_a, outcome_b);
        assert_eq!(outcome_a, Some(BattleOutcome::MutualWithdrawal));
    }
}
