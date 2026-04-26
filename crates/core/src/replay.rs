//! Replay support (PROMPT.md §16.18).
//!
//! A replay is a deterministic fold of ordered [`Event`] values over an
//! initial [`Scenario`]. This module intentionally performs read-only
//! state reconstruction from the event log instead of invoking the full
//! mutating resolvers.

use gc1805_core_schema::{
    canonical::canonical_hash,
    events::{Event, MovementResolved},
    scenario::Scenario,
};
use serde::{Deserialize, Serialize};

pub const REPLAY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFile {
    pub schema_version: u32,
    pub game_id: String,
    pub initial_scenario: Scenario,
    pub events: Vec<Event>,
    /// Hashes of the reconstructed scenario after each seekable turn.
    pub turn_hashes: Vec<TurnHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnHash {
    pub turn: u32,
    /// 64-character hex BLAKE3 of the canonical scenario state.
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct ReplayPlayer {
    pub file: ReplayFile,
    pub current_turn: usize,
}

pub fn create_replay(game_id: &str, initial_scenario: &Scenario) -> ReplayFile {
    ReplayFile {
        schema_version: REPLAY_SCHEMA_VERSION,
        game_id: game_id.to_owned(),
        initial_scenario: initial_scenario.clone(),
        events: Vec::new(),
        turn_hashes: Vec::new(),
    }
}

pub fn append_events(
    replay: &mut ReplayFile,
    events: Vec<Event>,
    turn: u32,
    scenario_after: &Scenario,
) {
    replay.events.extend(events);
    let hash = canonical_hash(scenario_after)
        .expect("scenario_after must serialize to canonical JSON for replay hashing");
    replay.turn_hashes.push(TurnHash { turn, hash });
}

pub fn seek_to_turn(replay: &ReplayFile, target_turn: u32) -> Result<Scenario, String> {
    if target_turn == 0 {
        return Ok(replay.initial_scenario.clone());
    }

    if !replay
        .turn_hashes
        .iter()
        .any(|entry| entry.turn == target_turn)
    {
        return Err(format!("unknown replay turn {target_turn}"));
    }

    let mut scenario = replay.initial_scenario.clone();
    for event in &replay.events {
        apply_event(&mut scenario, event)?;
        if matches!(event, Event::TurnCompleted { turn, .. } if *turn + 1 == target_turn) {
            return Ok(scenario);
        }
    }

    Err(format!(
        "replay ended before reaching turn {target_turn}; missing TurnCompleted marker"
    ))
}

pub fn verify_integrity(replay: &ReplayFile) -> Result<(), String> {
    for turn_hash in &replay.turn_hashes {
        let scenario = seek_to_turn(replay, turn_hash.turn)?;
        let actual = canonical_hash(&scenario).map_err(|err| err.to_string())?;
        if actual != turn_hash.hash {
            return Err(format!(
                "integrity check failed at turn {}: expected {}, got {}",
                turn_hash.turn, turn_hash.hash, actual
            ));
        }
    }

    Ok(())
}

pub fn save_replay(replay: &ReplayFile) -> Result<String, String> {
    serde_json::to_string_pretty(replay).map_err(|err| err.to_string())
}

pub fn load_replay(json: &str) -> Result<ReplayFile, String> {
    serde_json::from_str(json).map_err(|err| err.to_string())
}

fn apply_event(scenario: &mut Scenario, event: &Event) -> Result<(), String> {
    match event {
        Event::IncomePaid { power, net, .. } => {
            let state = scenario
                .power_state
                .get_mut(power)
                .ok_or_else(|| format!("missing power state for {}", power.as_str()))?;
            state.treasury += net;
        }
        Event::MaintenancePaid {
            power,
            corps_cost,
            fleet_cost,
        } => {
            let state = scenario
                .power_state
                .get_mut(power)
                .ok_or_else(|| format!("missing power state for {}", power.as_str()))?;
            state.treasury -= corps_cost + fleet_cost;
        }
        Event::TreasuryInDeficit { power, .. } => {
            let state = scenario
                .power_state
                .get_mut(power)
                .ok_or_else(|| format!("missing power state for {}", power.as_str()))?;
            state.treasury = 0;
        }
        Event::MovementResolved(MovementResolved { corps, to, .. }) => {
            let corps_state = scenario
                .corps
                .get_mut(corps)
                .ok_or_else(|| format!("missing corps {}", corps.as_str()))?;
            corps_state.area = to.clone();
        }
        Event::TurnCompleted { turn, .. } => {
            scenario.current_turn = turn + 1;
        }
        Event::ForcedMarchResolved(_)
        | Event::InterceptionQueued(_)
        | Event::OrderRejected(_)
        | Event::PhaseCompleted { .. }
        | Event::TurnStarted { .. }
        | Event::ReplacementsArrived { .. }
        | Event::UnitProduced { .. }
        | Event::SubsidyTransferred { .. }
        | Event::TaxPolicySet { .. }
        | Event::BattleResolved { .. }
        | Event::CorpsRetreated { .. }
        | Event::CorpsRouted { .. }
        | Event::LeaderCasualty { .. }
        | Event::SupplyTraced { .. }
        | Event::AttritionApplied { .. }
        | Event::WarDeclared { .. }
        | Event::PeaceProposed { .. }
        | Event::PeaceAccepted { .. }
        | Event::AllianceFormed { .. }
        | Event::AllianceBroken { .. }
        | Event::PrestigeChanged { .. }
        | Event::AllianceCascade { .. }
        | Event::PrestigeAwarded { .. }
        | Event::RevoltTriggered { .. }
        | Event::PeaceConferenceOpened { .. }
        | Event::AbdicationForced { .. }
        | Event::MinorActivated { .. }
        | Event::MinorRevolt { .. }
        | Event::FleetMoved { .. }
        | Event::FleetEnteredPort { .. }
        | Event::FleetLeftPort { .. }
        | Event::NavalBattleResolved { .. }
        | Event::BlockadeEstablished { .. }
        | Event::CorpsEmbarked { .. }
        | Event::CorpsDisembarked { .. } => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::{
        canonical::canonical_hash,
        events::{MovementResolved, OrderRejected},
        ids::{AreaId, CorpsId, FleetId, LeaderId, MinorId, PowerId, SeaZoneId},
        scenario::{
            Area, AreaAdjacency, CoastLink, Corps, DiplomaticPairKey, DiplomaticState, Features,
            Fleet, GameDate, Leader, MinorSetup, MovementRules, Owner, PendingSubsidy, PowerSetup,
            PowerSlot, PowerState, ProductionItem, ReplacementItem, Scenario, SeaAdjacency,
            SeaZone, TaxPolicy, Terrain, SCHEMA_VERSION,
        },
        tables::Maybe,
    };
    use std::collections::BTreeMap;

    fn test_scenario() -> Scenario {
        let fra = PowerId::from("FRA");
        let aus = PowerId::from("AUS");
        let area_paris = AreaId::from("AREA_PARIS");
        let area_lyon = AreaId::from("AREA_LYON");
        let napoleon = LeaderId::from("LEADER_NAPOLEON");
        let francis = LeaderId::from("LEADER_FRANCIS");
        let corps_id = CorpsId::from("CORPS_FRA_001");

        let mut power_state = BTreeMap::new();
        power_state.insert(
            fra.clone(),
            PowerState {
                treasury: 10,
                manpower: 5,
                prestige: 3,
                tax_policy: TaxPolicy::Standard,
            },
        );
        power_state.insert(
            aus.clone(),
            PowerState {
                treasury: 7,
                manpower: 4,
                prestige: 2,
                tax_policy: TaxPolicy::Low,
            },
        );

        let mut powers = BTreeMap::new();
        powers.insert(
            fra.clone(),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: napoleon.clone(),
                capital: area_paris.clone(),
                starting_treasury: 10,
                starting_manpower: 5,
                starting_pp: 3,
                max_corps: 10,
                max_depots: 5,
                mobilization_areas: vec![area_paris.clone()],
                color_hex: "#123456".into(),
            },
        );
        powers.insert(
            aus.clone(),
            PowerSetup {
                display_name: "Austria".into(),
                house: "Habsburg".into(),
                ruler: francis.clone(),
                capital: area_lyon.clone(),
                starting_treasury: 7,
                starting_manpower: 4,
                starting_pp: 2,
                max_corps: 10,
                max_depots: 5,
                mobilization_areas: vec![area_lyon.clone()],
                color_hex: "#654321".into(),
            },
        );

        let mut leaders = BTreeMap::new();
        leaders.insert(
            napoleon.clone(),
            Leader {
                display_name: "Napoleon".into(),
                strategic: 6,
                tactical: 6,
                initiative: 6,
                army_commander: true,
                born: GameDate::new(1769, 8),
            },
        );
        leaders.insert(
            francis,
            Leader {
                display_name: "Francis".into(),
                strategic: 2,
                tactical: 1,
                initiative: 1,
                army_commander: true,
                born: GameDate::new(1768, 2),
            },
        );

        let mut areas = BTreeMap::new();
        areas.insert(
            area_paris.clone(),
            Area {
                display_name: "Paris".into(),
                owner: Owner::Power(PowerSlot { power: fra.clone() }),
                terrain: Terrain::Urban,
                fort_level: 2,
                money_yield: Maybe::Value(5),
                manpower_yield: Maybe::Value(2),
                capital_of: Some(fra.clone()),
                port: false,
                blockaded: false,
                map_x: 100,
                map_y: 100,
            },
        );
        areas.insert(
            area_lyon.clone(),
            Area {
                display_name: "Lyon".into(),
                owner: Owner::Power(PowerSlot { power: fra.clone() }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: Maybe::Value(3),
                manpower_yield: Maybe::Value(1),
                capital_of: None,
                port: false,
                blockaded: false,
                map_x: 120,
                map_y: 130,
            },
        );

        let mut corps = BTreeMap::new();
        corps.insert(
            corps_id,
            Corps {
                display_name: "I Corps".into(),
                owner: fra,
                area: area_paris,
                infantry_sp: 5,
                cavalry_sp: 1,
                artillery_sp: 1,
                morale_q4: 8000,
                supplied: true,
                leader: Some(napoleon),
            },
        );

        Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 1,
            scenario_id: "replay_test".into(),
            name: "Replay Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: false,
            features: Features::default(),
            movement_rules: MovementRules::default(),
            current_turn: 0,
            power_state,
            production_queue: Vec::<ProductionItem>::new(),
            replacement_queue: Vec::<ReplacementItem>::new(),
            subsidy_queue: Vec::<PendingSubsidy>::new(),
            powers,
            minors: BTreeMap::<MinorId, MinorSetup>::new(),
            leaders,
            areas,
            sea_zones: BTreeMap::<SeaZoneId, SeaZone>::new(),
            corps,
            fleets: BTreeMap::<FleetId, Fleet>::new(),
            diplomacy: BTreeMap::<DiplomaticPairKey, DiplomaticState>::new(),
            adjacency: Vec::<AreaAdjacency>::new(),
            coast_links: Vec::<CoastLink>::new(),
            sea_adjacency: Vec::<SeaAdjacency>::new(),
        }
    }

    fn movement_event(from: &str, to: &str) -> Event {
        Event::MovementResolved(MovementResolved {
            corps: CorpsId::from("CORPS_FRA_001"),
            from: AreaId::from(from),
            to: AreaId::from(to),
            hops: 1,
            path: vec![AreaId::from(from), AreaId::from(to)],
        })
    }

    fn completed_turn(turn: u32) -> Event {
        Event::TurnCompleted {
            turn,
            state_hash: String::new(),
        }
    }

    #[test]
    fn create_replay_empty_events() {
        let scenario = test_scenario();
        let replay = create_replay("game-1", &scenario);

        assert_eq!(replay.schema_version, REPLAY_SCHEMA_VERSION);
        assert_eq!(replay.game_id, "game-1");
        assert!(replay.events.is_empty());
        assert!(replay.turn_hashes.is_empty());
    }

    #[test]
    fn append_events_grows_event_list() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut next = scenario.clone();
        next.current_turn = 1;

        append_events(&mut replay, vec![completed_turn(0)], 1, &next);

        assert_eq!(replay.events.len(), 1);
    }

    #[test]
    fn append_events_records_hash() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut next = scenario.clone();
        next.current_turn = 1;

        append_events(&mut replay, vec![completed_turn(0)], 1, &next);

        assert_eq!(replay.turn_hashes.len(), 1);
        assert_eq!(replay.turn_hashes[0].turn, 1);
        assert_eq!(replay.turn_hashes[0].hash, canonical_hash(&next).unwrap());
    }

    #[test]
    fn seek_to_turn_zero_is_initial() {
        let scenario = test_scenario();
        let replay = create_replay("game-1", &scenario);

        let sought = seek_to_turn(&replay, 0).unwrap();

        assert_eq!(
            canonical_hash(&sought).unwrap(),
            canonical_hash(&scenario).unwrap()
        );
    }

    #[test]
    fn seek_to_turn_applies_income_event() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after
            .power_state
            .get_mut(&PowerId::from("FRA"))
            .unwrap()
            .treasury += 8;
        after.current_turn = 1;

        append_events(
            &mut replay,
            vec![
                Event::IncomePaid {
                    power: PowerId::from("FRA"),
                    gross: 10,
                    net: 8,
                    tax_policy: TaxPolicy::Standard,
                },
                completed_turn(0),
            ],
            1,
            &after,
        );

        let sought = seek_to_turn(&replay, 1).unwrap();
        assert_eq!(sought.power_state[&PowerId::from("FRA")].treasury, 18);
        assert_eq!(sought.current_turn, 1);
    }

    #[test]
    fn seek_to_turn_applies_movement_event() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after
            .corps
            .get_mut(&CorpsId::from("CORPS_FRA_001"))
            .unwrap()
            .area = AreaId::from("AREA_LYON");
        after.current_turn = 1;

        append_events(
            &mut replay,
            vec![movement_event("AREA_PARIS", "AREA_LYON"), completed_turn(0)],
            1,
            &after,
        );

        let sought = seek_to_turn(&replay, 1).unwrap();
        assert_eq!(
            sought.corps[&CorpsId::from("CORPS_FRA_001")].area.as_str(),
            "AREA_LYON"
        );
    }

    #[test]
    fn seek_to_turn_unknown_turn_returns_err() {
        let scenario = test_scenario();
        let replay = create_replay("game-1", &scenario);

        let err = seek_to_turn(&replay, 2).unwrap_err();
        assert!(err.contains("unknown replay turn 2"));
    }

    #[test]
    fn verify_integrity_clean_replay_ok() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);

        let mut after_turn_one = scenario.clone();
        after_turn_one
            .power_state
            .get_mut(&PowerId::from("FRA"))
            .unwrap()
            .treasury += 8;
        after_turn_one.current_turn = 1;
        append_events(
            &mut replay,
            vec![
                Event::IncomePaid {
                    power: PowerId::from("FRA"),
                    gross: 10,
                    net: 8,
                    tax_policy: TaxPolicy::Standard,
                },
                completed_turn(0),
            ],
            1,
            &after_turn_one,
        );

        assert!(verify_integrity(&replay).is_ok());
    }

    #[test]
    fn verify_integrity_tampered_hash_fails() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after.current_turn = 1;

        append_events(&mut replay, vec![completed_turn(0)], 1, &after);
        replay.turn_hashes[0].hash = "0".repeat(64);

        let err = verify_integrity(&replay).unwrap_err();
        assert!(err.contains("integrity check failed"));
    }

    #[test]
    fn save_and_load_round_trip() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after.current_turn = 1;
        append_events(&mut replay, vec![completed_turn(0)], 1, &after);

        let json = save_replay(&replay).unwrap();
        let loaded = load_replay(&json).unwrap();

        assert_eq!(loaded.game_id, replay.game_id);
        assert_eq!(loaded.events.len(), replay.events.len());
        assert_eq!(loaded.turn_hashes.len(), replay.turn_hashes.len());
    }

    #[test]
    fn load_invalid_json_returns_err() {
        let err = load_replay("{not valid json}").unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn turn_hashes_count_matches_turns() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);

        let mut after_one = scenario.clone();
        after_one.current_turn = 1;
        append_events(&mut replay, vec![completed_turn(0)], 1, &after_one);

        let mut after_two = after_one.clone();
        after_two.current_turn = 2;
        append_events(&mut replay, vec![completed_turn(1)], 2, &after_two);

        assert_eq!(replay.turn_hashes.len(), 2);
    }

    #[test]
    fn seek_intermediate_turn() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);

        let mut after_one = scenario.clone();
        after_one
            .power_state
            .get_mut(&PowerId::from("FRA"))
            .unwrap()
            .treasury += 8;
        after_one.current_turn = 1;
        append_events(
            &mut replay,
            vec![
                Event::IncomePaid {
                    power: PowerId::from("FRA"),
                    gross: 10,
                    net: 8,
                    tax_policy: TaxPolicy::Standard,
                },
                completed_turn(0),
            ],
            1,
            &after_one,
        );

        let mut after_two = after_one.clone();
        after_two
            .corps
            .get_mut(&CorpsId::from("CORPS_FRA_001"))
            .unwrap()
            .area = AreaId::from("AREA_LYON");
        after_two.current_turn = 2;
        append_events(
            &mut replay,
            vec![movement_event("AREA_PARIS", "AREA_LYON"), completed_turn(1)],
            2,
            &after_two,
        );

        let sought = seek_to_turn(&replay, 1).unwrap();
        assert_eq!(sought.current_turn, 1);
        assert_eq!(
            sought.corps[&CorpsId::from("CORPS_FRA_001")].area.as_str(),
            "AREA_PARIS"
        );
        assert_eq!(sought.power_state[&PowerId::from("FRA")].treasury, 18);
    }

    #[test]
    fn replay_deterministic() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after
            .power_state
            .get_mut(&PowerId::from("FRA"))
            .unwrap()
            .treasury += 8;
        after.current_turn = 1;
        append_events(
            &mut replay,
            vec![
                Event::IncomePaid {
                    power: PowerId::from("FRA"),
                    gross: 10,
                    net: 8,
                    tax_policy: TaxPolicy::Standard,
                },
                completed_turn(0),
            ],
            1,
            &after,
        );

        let a = seek_to_turn(&replay, 1).unwrap();
        let b = seek_to_turn(&replay, 1).unwrap();

        assert_eq!(canonical_hash(&a).unwrap(), canonical_hash(&b).unwrap());
    }

    #[test]
    fn seek_applies_maintenance_and_deficit() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after
            .power_state
            .get_mut(&PowerId::from("FRA"))
            .unwrap()
            .treasury = 0;
        after.current_turn = 1;

        append_events(
            &mut replay,
            vec![
                Event::MaintenancePaid {
                    power: PowerId::from("FRA"),
                    corps_cost: 6,
                    fleet_cost: 7,
                },
                Event::TreasuryInDeficit {
                    power: PowerId::from("FRA"),
                    shortfall: 3,
                },
                completed_turn(0),
            ],
            1,
            &after,
        );

        let sought = seek_to_turn(&replay, 1).unwrap();
        assert_eq!(sought.power_state[&PowerId::from("FRA")].treasury, 0);
    }

    #[test]
    fn seek_skips_unhandled_events() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after.current_turn = 1;

        append_events(
            &mut replay,
            vec![
                Event::OrderRejected(OrderRejected {
                    reason_code: "X".into(),
                    message: "ignored".into(),
                }),
                completed_turn(0),
            ],
            1,
            &after,
        );

        assert!(seek_to_turn(&replay, 1).is_ok());
    }

    #[test]
    fn verify_integrity_requires_turn_completed_marker() {
        let scenario = test_scenario();
        let mut replay = create_replay("game-1", &scenario);
        let mut after = scenario.clone();
        after.current_turn = 1;
        append_events(
            &mut replay,
            vec![Event::IncomePaid {
                power: PowerId::from("FRA"),
                gross: 10,
                net: 8,
                tax_policy: TaxPolicy::Standard,
            }],
            1,
            &after,
        );

        let err = verify_integrity(&replay).unwrap_err();
        assert!(err.contains("missing TurnCompleted marker"));
    }
}
