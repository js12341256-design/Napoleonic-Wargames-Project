#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use gc1805_core::orders::{BuildCorpsOrder, DeclareWarOrder, HoldOrder, MoveOrder, Order};
use gc1805_core::{validate_order, MapGraph};
use gc1805_core_schema::{
    ids::{AreaId, CorpsId, PowerId},
    scenario::{CorpsComposition, DiplomaticPairKey, DiplomaticState, Scenario},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AiPersonality {
    pub id: String,
    pub aggression: u8,
    pub defensiveness: u8,
    pub diplomatic_openness: u8,
    pub economic_priority: u8,
}

impl Default for AiPersonality {
    fn default() -> Self {
        Self {
            id: "DEFAULT".into(),
            aggression: 5,
            defensiveness: 5,
            diplomatic_openness: 5,
            economic_priority: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiContext<'a> {
    pub projected: &'a Scenario,
    pub power: PowerId,
    pub personality: AiPersonality,
    pub rng_seed: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AiOrders {
    pub movement_orders: Vec<Order>,
    pub economic_orders: Vec<Order>,
    pub diplomatic_orders: Vec<Order>,
}

fn str_hash(s: &str) -> u64 {
    s.bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

fn adjacent_areas(scenario: &Scenario, area: &AreaId) -> Vec<AreaId> {
    let mut result: Vec<AreaId> = scenario
        .adjacency
        .iter()
        .filter_map(|adj| {
            if &adj.from == area {
                Some(adj.to.clone())
            } else if &adj.to == area {
                Some(adj.from.clone())
            } else {
                None
            }
        })
        .collect();
    result.sort();
    result.dedup();
    result
}

fn has_enemy_corps(scenario: &Scenario, area: &AreaId, own_power: &PowerId) -> bool {
    scenario
        .corps
        .values()
        .any(|c| &c.owner != own_power && &c.area == area)
}

fn enemy_adjacent(scenario: &Scenario, area: &AreaId, own_power: &PowerId) -> bool {
    adjacent_areas(scenario, area)
        .iter()
        .any(|a| has_enemy_corps(scenario, a, own_power))
}

fn distance_to_capital(scenario: &Scenario, from: &AreaId, capital: &AreaId) -> Option<usize> {
    MapGraph::from_scenario(scenario)
        .shortest_path_hops(from, capital)
        .map(|path| path.len())
}

fn default_build_composition(scenario: &Scenario, power: &PowerId) -> Option<CorpsComposition> {
    scenario
        .corps
        .iter()
        .find(|(_, corps)| &corps.owner == power)
        .map(|(_, corps)| CorpsComposition {
            infantry_sp: corps.infantry_sp,
            cavalry_sp: corps.cavalry_sp,
            artillery_sp: corps.artillery_sp,
        })
}

fn pair_contains_power(key: &DiplomaticPairKey, power: &PowerId) -> bool {
    &key.0 == power || &key.1 == power
}

fn other_power(key: &DiplomaticPairKey, power: &PowerId) -> Option<PowerId> {
    if &key.0 == power {
        Some(key.1.clone())
    } else if &key.1 == power {
        Some(key.0.clone())
    } else {
        None
    }
}

fn make_hold_order(power: &PowerId, corps_id: &CorpsId, scenario: &Scenario) -> Option<Order> {
    let order = Order::Hold(HoldOrder {
        submitter: power.clone(),
        corps: corps_id.clone(),
    });
    validate_order(scenario, &order).ok().map(|_| order)
}

fn make_move_order(
    power: &PowerId,
    corps_id: &CorpsId,
    destination: &AreaId,
    scenario: &Scenario,
) -> Option<Order> {
    let order = Order::Move(MoveOrder {
        submitter: power.clone(),
        corps: corps_id.clone(),
        to: destination.clone(),
    });
    validate_order(scenario, &order).ok().map(|_| order)
}

fn make_build_corps_order(power: &PowerId, area: &AreaId, scenario: &Scenario) -> Option<Order> {
    let composition = default_build_composition(scenario, power)?;
    Some(Order::BuildCorps(BuildCorpsOrder {
        submitter: power.clone(),
        area: area.clone(),
        composition,
    }))
}

fn make_declare_war_order(power: &PowerId, target: &PowerId) -> Option<Order> {
    if power == target {
        return None;
    }
    Some(Order::DeclareWar(DeclareWarOrder {
        submitter: power.clone(),
        target: target.clone(),
    }))
}

pub fn generate_orders(ctx: &AiContext<'_>) -> AiOrders {
    let mut orders = AiOrders::default();
    let scenario = ctx.projected;
    let power = &ctx.power;
    let capital = scenario.powers.get(power).map(|p| p.capital.clone());

    for (corps_id, corps) in &scenario.corps {
        if &corps.owner != power {
            continue;
        }

        if enemy_adjacent(scenario, &corps.area, power) {
            if let Some(order) = make_hold_order(power, corps_id, scenario) {
                orders.movement_orders.push(order);
            }
            continue;
        }

        if let Some(ref cap) = capital {
            if &corps.area == cap {
                continue;
            }

            let candidates: Vec<(AreaId, usize)> = adjacent_areas(scenario, &corps.area)
                .into_iter()
                .filter_map(|area| {
                    distance_to_capital(scenario, &area, cap).map(|distance| (area, distance))
                })
                .collect();

            let target = candidates
                .iter()
                .map(|(_, distance)| *distance)
                .min()
                .and_then(|best_distance| {
                    let mut tied: Vec<AreaId> = candidates
                        .iter()
                        .filter(|(_, distance)| *distance == best_distance)
                        .map(|(area, _)| area.clone())
                        .collect();
                    tied.sort_by_key(|area| (str_hash(area.as_str()), area.clone()));
                    if tied.is_empty() {
                        None
                    } else {
                        Some(tied[(ctx.rng_seed as usize) % tied.len()].clone())
                    }
                });

            if let Some(destination) = target {
                if let Some(order) = make_move_order(power, corps_id, &destination, scenario) {
                    orders.movement_orders.push(order);
                }
            }
        }
    }

    if let Some(ps) = scenario.power_state.get(power) {
        let threshold = if ctx.personality.economic_priority >= 7 {
            50
        } else {
            100
        };
        if ps.treasury > threshold && ps.manpower > 20 && scenario.production_queue.is_empty() {
            if let Some(ref cap) = capital {
                if let Some(order) = make_build_corps_order(power, cap, scenario) {
                    orders.economic_orders.push(order);
                }
            }
        }
    }

    if ctx.personality.aggression >= 8 {
        for (key, state) in &scenario.diplomacy {
            if *state != DiplomaticState::Unfriendly || !pair_contains_power(key, power) {
                continue;
            }
            let Some(other) = other_power(key, power) else {
                continue;
            };
            let war_key = DiplomaticPairKey::new(power.clone(), other.clone());
            if scenario.diplomacy.get(&war_key) == Some(&DiplomaticState::War) {
                continue;
            }
            if let Some(order) = make_declare_war_order(power, &other) {
                orders.diplomatic_orders.push(order);
                break;
            }
        }
    }

    orders
}

pub fn personality_from_map(fields: &BTreeMap<String, u8>) -> AiPersonality {
    AiPersonality {
        id: "DEFAULT".into(),
        aggression: *fields.get("aggression").unwrap_or(&5),
        defensiveness: *fields.get("defensiveness").unwrap_or(&5),
        diplomatic_openness: *fields.get("diplomatic_openness").unwrap_or(&5),
        economic_priority: *fields.get("economic_priority").unwrap_or(&5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::LeaderId;
    use gc1805_core_schema::scenario::{
        Area, AreaAdjacency, Corps, Features, GameDate, Leader, MovementRules, Owner, PowerSetup,
        PowerSlot, PowerState, TaxPolicy, Terrain, SCHEMA_VERSION,
    };
    use gc1805_core_schema::tables::Maybe;

    fn fra() -> PowerId {
        PowerId::from("FRA")
    }

    fn aus() -> PowerId {
        PowerId::from("AUS")
    }

    fn pru() -> PowerId {
        PowerId::from("PRU")
    }

    fn area(id: &str) -> AreaId {
        AreaId::from(id)
    }

    fn corps(id: &str) -> CorpsId {
        CorpsId::from(id)
    }

    fn leader(id: &str) -> LeaderId {
        LeaderId::from(id)
    }

    fn default_personality() -> AiPersonality {
        AiPersonality::default()
    }

    fn aggressive_personality() -> AiPersonality {
        AiPersonality {
            aggression: 8,
            ..AiPersonality::default()
        }
    }

    fn fixture() -> Scenario {
        let mut scenario = Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "ai-test".into(),
            name: "AI Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1805, 5),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules {
                max_corps_per_area: Maybe::Value(3),
                movement_hops_per_turn: Maybe::Value(1),
                forced_march_extra_hops: Maybe::Value(1),
                forced_march_morale_loss_q4: Maybe::Value(500),
            },
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
            leader("LEADER_NAPOLEON"),
            Leader {
                display_name: "Napoleon".into(),
                strategic: 6,
                tactical: 6,
                initiative: 6,
                army_commander: true,
                born: GameDate::new(1769, 8),
            },
        );
        scenario.leaders.insert(
            leader("LEADER_CHARLES"),
            Leader {
                display_name: "Charles".into(),
                strategic: 5,
                tactical: 5,
                initiative: 5,
                army_commander: true,
                born: GameDate::new(1771, 9),
            },
        );

        for area_id in [
            "AREA_CAPITAL",
            "AREA_MID",
            "AREA_FRONT",
            "AREA_ENEMY_BORDER",
            "AREA_TIE_A",
            "AREA_TIE_B",
            "AREA_ISOLATED",
        ] {
            scenario.areas.insert(
                area(area_id),
                Area {
                    display_name: area_id.into(),
                    owner: Owner::Power(PowerSlot { power: fra() }),
                    terrain: Terrain::Open,
                    fort_level: 0,
                    money_yield: Maybe::Value(1),
                    manpower_yield: Maybe::Value(1),
                    capital_of: (area_id == "AREA_CAPITAL").then_some(fra()),
                    port: false,
                    blockaded: false,
                    map_x: 0,
                    map_y: 0,
                },
            );
        }

        for (from, to) in [
            ("AREA_CAPITAL", "AREA_MID"),
            ("AREA_MID", "AREA_FRONT"),
            ("AREA_FRONT", "AREA_ENEMY_BORDER"),
            ("AREA_FRONT", "AREA_TIE_A"),
            ("AREA_FRONT", "AREA_TIE_B"),
        ] {
            scenario.adjacency.push(AreaAdjacency {
                from: area(from),
                to: area(to),
                cost: Maybe::Value(1),
            });
            scenario.adjacency.push(AreaAdjacency {
                from: area(to),
                to: area(from),
                cost: Maybe::Value(1),
            });
        }

        scenario.powers.insert(
            fra(),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: leader("LEADER_NAPOLEON"),
                capital: area("AREA_CAPITAL"),
                starting_treasury: 120,
                starting_manpower: 40,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 8,
                mobilization_areas: vec![area("AREA_CAPITAL")],
                color_hex: "#123456".into(),
            },
        );
        scenario.powers.insert(
            aus(),
            PowerSetup {
                display_name: "Austria".into(),
                house: "Habsburg".into(),
                ruler: leader("LEADER_CHARLES"),
                capital: area("AREA_ENEMY_BORDER"),
                starting_treasury: 80,
                starting_manpower: 40,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 8,
                mobilization_areas: vec![area("AREA_ENEMY_BORDER")],
                color_hex: "#654321".into(),
            },
        );
        scenario.powers.insert(
            pru(),
            PowerSetup {
                display_name: "Prussia".into(),
                house: "Hohenzollern".into(),
                ruler: leader("LEADER_CHARLES"),
                capital: area("AREA_ISOLATED"),
                starting_treasury: 80,
                starting_manpower: 40,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 8,
                mobilization_areas: vec![area("AREA_ISOLATED")],
                color_hex: "#abcdef".into(),
            },
        );

        scenario.power_state.insert(
            fra(),
            PowerState {
                treasury: 120,
                manpower: 40,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        scenario.power_state.insert(
            aus(),
            PowerState {
                treasury: 80,
                manpower: 40,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        scenario.power_state.insert(
            pru(),
            PowerState {
                treasury: 80,
                manpower: 40,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );

        scenario.corps.insert(
            corps("CORPS_FRA_001"),
            Corps {
                display_name: "I Corps".into(),
                owner: fra(),
                area: area("AREA_FRONT"),
                infantry_sp: 4,
                cavalry_sp: 1,
                artillery_sp: 1,
                morale_q4: 9000,
                supplied: true,
                leader: Some(leader("LEADER_NAPOLEON")),
            },
        );

        scenario
    }

    fn context<'a>(scenario: &'a Scenario) -> AiContext<'a> {
        AiContext {
            projected: scenario,
            power: fra(),
            personality: default_personality(),
            rng_seed: 7,
        }
    }

    #[test]
    fn str_hash_is_deterministic() {
        assert_eq!(str_hash("AREA_CAPITAL"), str_hash("AREA_CAPITAL"));
    }

    #[test]
    fn adjacent_areas_are_sorted_and_deduped() {
        let mut scenario = fixture();
        scenario.adjacency.push(AreaAdjacency {
            from: area("AREA_FRONT"),
            to: area("AREA_MID"),
            cost: Maybe::Value(1),
        });
        let areas = adjacent_areas(&scenario, &area("AREA_FRONT"));
        assert_eq!(areas[0], area("AREA_ENEMY_BORDER"));
        assert_eq!(areas[1], area("AREA_MID"));
        assert_eq!(areas.len(), 4);
    }

    #[test]
    fn detects_enemy_corps_in_area() {
        let mut scenario = fixture();
        scenario.corps.insert(
            corps("CORPS_AUS_001"),
            Corps {
                display_name: "Austrian".into(),
                owner: aus(),
                area: area("AREA_MID"),
                infantry_sp: 4,
                cavalry_sp: 0,
                artillery_sp: 0,
                morale_q4: 8000,
                supplied: true,
                leader: None,
            },
        );
        assert!(has_enemy_corps(&scenario, &area("AREA_MID"), &fra()));
        assert!(!has_enemy_corps(&scenario, &area("AREA_MID"), &aus()));
    }

    #[test]
    fn detects_enemy_adjacent() {
        let mut scenario = fixture();
        scenario.corps.insert(
            corps("CORPS_AUS_001"),
            Corps {
                display_name: "Austrian".into(),
                owner: aus(),
                area: area("AREA_MID"),
                infantry_sp: 4,
                cavalry_sp: 0,
                artillery_sp: 0,
                morale_q4: 8000,
                supplied: true,
                leader: None,
            },
        );
        assert!(enemy_adjacent(&scenario, &area("AREA_FRONT"), &fra()));
    }

    #[test]
    fn hold_order_helper_builds_valid_hold() {
        let scenario = fixture();
        let order = make_hold_order(&fra(), &corps("CORPS_FRA_001"), &scenario).unwrap();
        assert!(matches!(order, Order::Hold(_)));
    }

    #[test]
    fn move_order_helper_rejects_illegal_move() {
        let scenario = fixture();
        let order = make_move_order(
            &fra(),
            &corps("CORPS_FRA_001"),
            &area("AREA_CAPITAL"),
            &scenario,
        );
        assert!(order.is_none());
    }

    #[test]
    fn move_order_helper_accepts_legal_move() {
        let scenario = fixture();
        let order = make_move_order(
            &fra(),
            &corps("CORPS_FRA_001"),
            &area("AREA_MID"),
            &scenario,
        );
        assert!(matches!(order, Some(Order::Move(_))));
    }

    #[test]
    fn build_corps_helper_uses_existing_composition() {
        let scenario = fixture();
        let order = make_build_corps_order(&fra(), &area("AREA_CAPITAL"), &scenario).unwrap();
        match order {
            Order::BuildCorps(o) => {
                assert_eq!(o.composition.infantry_sp, 4);
                assert_eq!(o.composition.cavalry_sp, 1);
                assert_eq!(o.composition.artillery_sp, 1);
            }
            _ => panic!("expected build corps order"),
        }
    }

    #[test]
    fn declare_war_helper_rejects_self_target() {
        assert!(make_declare_war_order(&fra(), &fra()).is_none());
    }

    #[test]
    fn declare_war_helper_builds_order() {
        let order = make_declare_war_order(&fra(), &aus()).unwrap();
        assert!(matches!(order, Order::DeclareWar(_)));
    }

    #[test]
    fn generates_hold_when_enemy_adjacent() {
        let mut scenario = fixture();
        scenario.corps.insert(
            corps("CORPS_AUS_001"),
            Corps {
                display_name: "Austrian".into(),
                owner: aus(),
                area: area("AREA_MID"),
                infantry_sp: 4,
                cavalry_sp: 0,
                artillery_sp: 0,
                morale_q4: 8000,
                supplied: true,
                leader: None,
            },
        );
        let orders = generate_orders(&context(&scenario));
        assert!(matches!(orders.movement_orders[0], Order::Hold(_)));
    }

    #[test]
    fn moves_toward_capital_when_safe() {
        let scenario = fixture();
        let orders = generate_orders(&context(&scenario));
        match &orders.movement_orders[0] {
            Order::Move(o) => assert_eq!(o.to, area("AREA_MID")),
            _ => panic!("expected move order"),
        }
    }

    #[test]
    fn corps_at_capital_does_not_move() {
        let mut scenario = fixture();
        scenario
            .corps
            .get_mut(&corps("CORPS_FRA_001"))
            .unwrap()
            .area = area("AREA_CAPITAL");
        let orders = generate_orders(&context(&scenario));
        assert!(orders.movement_orders.is_empty());
    }

    #[test]
    fn unreachable_neighbors_are_ignored() {
        let mut scenario = fixture();
        scenario
            .corps
            .get_mut(&corps("CORPS_FRA_001"))
            .unwrap()
            .area = area("AREA_ISOLATED");
        let orders = generate_orders(&context(&scenario));
        assert!(orders.movement_orders.is_empty());
    }

    #[test]
    fn richer_ai_builds_corps_when_queue_empty() {
        let scenario = fixture();
        let orders = generate_orders(&context(&scenario));
        assert!(matches!(orders.economic_orders[0], Order::BuildCorps(_)));
    }

    #[test]
    fn high_economic_priority_uses_lower_threshold() {
        let mut scenario = fixture();
        scenario.power_state.get_mut(&fra()).unwrap().treasury = 60;
        let ctx = AiContext {
            projected: &scenario,
            power: fra(),
            personality: AiPersonality {
                economic_priority: 7,
                ..AiPersonality::default()
            },
            rng_seed: 7,
        };
        let orders = generate_orders(&ctx);
        assert_eq!(orders.economic_orders.len(), 1);
    }

    #[test]
    fn normal_economic_priority_skips_at_sixty_treasury() {
        let mut scenario = fixture();
        scenario.power_state.get_mut(&fra()).unwrap().treasury = 60;
        let orders = generate_orders(&context(&scenario));
        assert!(orders.economic_orders.is_empty());
    }

    #[test]
    fn skips_build_when_poor() {
        let mut scenario = fixture();
        scenario.power_state.get_mut(&fra()).unwrap().treasury = 20;
        let orders = generate_orders(&context(&scenario));
        assert!(orders.economic_orders.is_empty());
    }

    #[test]
    fn skips_build_when_manpower_low() {
        let mut scenario = fixture();
        scenario.power_state.get_mut(&fra()).unwrap().manpower = 20;
        let orders = generate_orders(&context(&scenario));
        assert!(orders.economic_orders.is_empty());
    }

    #[test]
    fn skips_build_when_production_queue_non_empty() {
        let mut scenario = fixture();
        scenario
            .production_queue
            .push(gc1805_core_schema::scenario::ProductionItem {
                owner: fra(),
                area: area("AREA_CAPITAL"),
                kind: gc1805_core_schema::scenario::ProductionKind::Corps,
                eta_turn: 1,
                corps_composition: None,
            });
        let orders = generate_orders(&context(&scenario));
        assert!(orders.economic_orders.is_empty());
    }

    #[test]
    fn aggressive_ai_declares_war_on_unfriendly_power() {
        let mut scenario = fixture();
        scenario.diplomacy.insert(
            DiplomaticPairKey::new(fra(), aus()),
            DiplomaticState::Unfriendly,
        );
        let ctx = AiContext {
            projected: &scenario,
            power: fra(),
            personality: aggressive_personality(),
            rng_seed: 7,
        };
        let orders = generate_orders(&ctx);
        assert!(matches!(orders.diplomatic_orders[0], Order::DeclareWar(_)));
    }

    #[test]
    fn non_aggressive_ai_skips_declare_war() {
        let mut scenario = fixture();
        scenario.diplomacy.insert(
            DiplomaticPairKey::new(fra(), aus()),
            DiplomaticState::Unfriendly,
        );
        let orders = generate_orders(&context(&scenario));
        assert!(orders.diplomatic_orders.is_empty());
    }

    #[test]
    fn ignores_unrelated_diplomatic_pairs() {
        let mut scenario = fixture();
        scenario.diplomacy.insert(
            DiplomaticPairKey::new(aus(), pru()),
            DiplomaticState::Unfriendly,
        );
        let ctx = AiContext {
            projected: &scenario,
            power: fra(),
            personality: aggressive_personality(),
            rng_seed: 7,
        };
        let orders = generate_orders(&ctx);
        assert!(orders.diplomatic_orders.is_empty());
    }

    #[test]
    fn declares_only_one_war_per_turn() {
        let mut scenario = fixture();
        scenario.diplomacy.insert(
            DiplomaticPairKey::new(fra(), aus()),
            DiplomaticState::Unfriendly,
        );
        scenario.diplomacy.insert(
            DiplomaticPairKey::new(fra(), pru()),
            DiplomaticState::Unfriendly,
        );
        let ctx = AiContext {
            projected: &scenario,
            power: fra(),
            personality: aggressive_personality(),
            rng_seed: 7,
        };
        let orders = generate_orders(&ctx);
        assert_eq!(orders.diplomatic_orders.len(), 1);
    }

    #[test]
    fn deterministic_for_same_seed() {
        let scenario = fixture();
        let ctx = context(&scenario);
        let a = generate_orders(&ctx);
        let b = generate_orders(&ctx);
        assert_eq!(
            serde_json::to_string(&a.movement_orders).unwrap(),
            serde_json::to_string(&b.movement_orders).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&a.economic_orders).unwrap(),
            serde_json::to_string(&b.economic_orders).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&a.diplomatic_orders).unwrap(),
            serde_json::to_string(&b.diplomatic_orders).unwrap()
        );
    }

    #[test]
    fn different_seed_changes_tie_break() {
        let mut scenario = fixture();
        scenario
            .corps
            .get_mut(&corps("CORPS_FRA_001"))
            .unwrap()
            .area = area("AREA_FRONT");
        scenario.adjacency.push(AreaAdjacency {
            from: area("AREA_TIE_A"),
            to: area("AREA_CAPITAL"),
            cost: Maybe::Value(1),
        });
        scenario.adjacency.push(AreaAdjacency {
            from: area("AREA_CAPITAL"),
            to: area("AREA_TIE_A"),
            cost: Maybe::Value(1),
        });
        scenario.adjacency.push(AreaAdjacency {
            from: area("AREA_TIE_B"),
            to: area("AREA_CAPITAL"),
            cost: Maybe::Value(1),
        });
        scenario.adjacency.push(AreaAdjacency {
            from: area("AREA_CAPITAL"),
            to: area("AREA_TIE_B"),
            cost: Maybe::Value(1),
        });
        let orders_a = generate_orders(&AiContext {
            projected: &scenario,
            power: fra(),
            personality: default_personality(),
            rng_seed: 1,
        });
        let orders_b = generate_orders(&AiContext {
            projected: &scenario,
            power: fra(),
            personality: default_personality(),
            rng_seed: 999,
        });
        let first_a = match &orders_a.movement_orders[0] {
            Order::Move(o) => o.to.clone(),
            _ => panic!("expected move"),
        };
        let first_b = match &orders_b.movement_orders[0] {
            Order::Move(o) => o.to.clone(),
            _ => panic!("expected move"),
        };
        assert_ne!(first_a, first_b);
    }

    #[test]
    fn personality_from_map_uses_defaults() {
        let fields = BTreeMap::new();
        let personality = personality_from_map(&fields);
        assert_eq!(personality, AiPersonality::default());
    }

    #[test]
    fn personality_from_map_overrides_present_fields() {
        let mut fields = BTreeMap::new();
        fields.insert("aggression".into(), 9);
        fields.insert("economic_priority".into(), 8);
        let personality = personality_from_map(&fields);
        assert_eq!(personality.aggression, 9);
        assert_eq!(personality.economic_priority, 8);
        assert_eq!(personality.defensiveness, 5);
    }
}
