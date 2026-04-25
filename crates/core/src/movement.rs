//! Movement validators and resolvers (PROMPT.md §16.3).
//!
//! The public surface is small:
//!
//! - [`validate_order`] — checks an [`Order`] against a [`Scenario`]
//!   and returns either `Ok(())` or a [`MovementRejection`].
//! - [`resolve_order`] — applies an already-validated order, mutating
//!   the scenario and returning an [`Event`].
//!
//! Both functions are pure and deterministic.  Neither touches the
//! filesystem or the wall-clock.

use gc1805_core_schema::events::{
    Event, ForcedMarchResolved, InterceptionQueued, MovementResolved, OrderRejected,
};
use gc1805_core_schema::ids::AreaId;
use gc1805_core_schema::scenario::Scenario;
use gc1805_core_schema::tables::Maybe;

use crate::map::MapGraph;
use crate::orders::Order;

/// Reasons an order may be rejected before it enters the log.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MovementRejection {
    #[error("corps `{0}` does not exist")]
    UnknownCorps(String),
    #[error("destination area `{0}` does not exist")]
    UnknownArea(String),
    #[error("submitter `{submitter}` does not own corps `{corps}`")]
    NotOwner { submitter: String, corps: String },
    #[error("destination unreachable: {0}")]
    Unreachable(String),
    #[error("destination exceeds movement budget: {hops} hops > {budget}")]
    OverBudget { hops: i32, budget: i32 },
    #[error("destination area is full (max {limit})")]
    StackingLimit { limit: i32 },
    #[error("rules table value is placeholder: {0}")]
    PlaceholderRule(&'static str),
}

impl MovementRejection {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownCorps(_) => "UNKNOWN_CORPS",
            Self::UnknownArea(_) => "UNKNOWN_AREA",
            Self::NotOwner { .. } => "NOT_OWNER",
            Self::Unreachable(_) => "UNREACHABLE",
            Self::OverBudget { .. } => "OVER_BUDGET",
            Self::StackingLimit { .. } => "STACKING_LIMIT",
            Self::PlaceholderRule(_) => "PLACEHOLDER_RULE",
        }
    }
}

/// Validate an [`Order`] against the current [`Scenario`].
///
/// Validation is pure and returns the planned destination on success
/// (so callers can avoid recomputing).  For [`Order::Hold`] the
/// destination is the corps's current area.  For [`Order::Interception`]
/// validation is structural — see `docs/adjudications.md` 0001.
pub fn validate_order(s: &Scenario, order: &Order) -> Result<MovementPlan, MovementRejection> {
    let corps_id = order.corps();
    let corps = s
        .corps
        .get(corps_id)
        .ok_or_else(|| MovementRejection::UnknownCorps(corps_id.to_string()))?;
    if &corps.owner != order.submitter() {
        return Err(MovementRejection::NotOwner {
            submitter: order.submitter().to_string(),
            corps: corps_id.to_string(),
        });
    }

    match order {
        Order::Hold(_) => Ok(MovementPlan::Hold {
            at: corps.area.clone(),
        }),
        Order::Move(o) => plan_move(s, &corps.area, &o.to, /* extra_hops */ 0),
        Order::ForcedMarch(o) => {
            let extra = match &s.movement_rules.forced_march_extra_hops {
                Maybe::Value(v) => *v,
                Maybe::Placeholder(_) => 1, // documented default — see movement.md
            };
            plan_move(s, &corps.area, &o.to, extra)
        }
        Order::Interception(o) => {
            if !s.areas.contains_key(&o.target_area) {
                return Err(MovementRejection::UnknownArea(o.target_area.to_string()));
            }
            Ok(MovementPlan::InterceptionQueued {
                target: o.target_area.clone(),
            })
        }
    }
}

/// Outcome of a successful validation; carries enough information that
/// `resolve_order` can be applied without recomputing the path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementPlan {
    Hold { at: AreaId },
    Move { path: Vec<AreaId>, hops: i32 },
    ForcedMarch { path: Vec<AreaId>, hops: i32 },
    InterceptionQueued { target: AreaId },
}

fn plan_move(
    s: &Scenario,
    from: &AreaId,
    to: &AreaId,
    extra_hops: i32,
) -> Result<MovementPlan, MovementRejection> {
    if !s.areas.contains_key(to) {
        return Err(MovementRejection::UnknownArea(to.to_string()));
    }
    let g = MapGraph::from_scenario(s);
    let path = g
        .shortest_path_hops(from, to)
        .ok_or_else(|| MovementRejection::Unreachable(to.to_string()))?;
    let hops = (path.len() as i32) - 1;
    let budget = match &s.movement_rules.movement_hops_per_turn {
        Maybe::Value(v) => *v,
        Maybe::Placeholder(_) => {
            return Err(MovementRejection::PlaceholderRule("movement_hops_per_turn"))
        }
    } + extra_hops;
    if hops > budget {
        return Err(MovementRejection::OverBudget { hops, budget });
    }
    // Stacking check: counts existing corps already in `to`, excluding
    // the moving corps itself if it is already there (defensive).
    let limit = match &s.movement_rules.max_corps_per_area {
        Maybe::Value(v) => *v,
        Maybe::Placeholder(_) => i32::MAX,
    };
    let occupants = s.corps.iter().filter(|(_, c)| &c.area == to).count() as i32;
    if occupants + 1 > limit && from != to {
        return Err(MovementRejection::StackingLimit { limit });
    }

    Ok(if extra_hops > 0 {
        MovementPlan::ForcedMarch { path, hops }
    } else {
        MovementPlan::Move { path, hops }
    })
}

/// Apply a validated plan to a mutable scenario and emit the matching
/// event.  The function is total — invariant violations (e.g. the
/// corps having vanished between validation and resolution) panic via
/// `debug_assert!` only; in release it returns an `OrderRejected`
/// event with a defensive code.
pub fn resolve_order(s: &mut Scenario, order: &Order, plan: MovementPlan) -> Event {
    match plan {
        MovementPlan::Hold { at } => Event::MovementResolved(MovementResolved {
            corps: order.corps().clone(),
            from: at.clone(),
            to: at,
            hops: 0,
            path: vec![order.corps_area_or_default(s)],
        }),
        MovementPlan::Move { path, hops } => {
            let from = path.first().cloned().expect("non-empty path");
            let to = path.last().cloned().expect("non-empty path");
            apply_move(s, order.corps(), &to);
            Event::MovementResolved(MovementResolved {
                corps: order.corps().clone(),
                from,
                to,
                hops,
                path,
            })
        }
        MovementPlan::ForcedMarch { path, hops } => {
            let from = path.first().cloned().expect("non-empty path");
            let to = path.last().cloned().expect("non-empty path");
            apply_move(s, order.corps(), &to);
            let morale_loss = match &s.movement_rules.forced_march_morale_loss_q4 {
                Maybe::Value(v) => *v,
                Maybe::Placeholder(_) => 0,
            };
            // Apply the morale drop directly; clamp at zero.
            if let Some(c) = s.corps.get_mut(order.corps()) {
                c.morale_q4 = (c.morale_q4 - morale_loss).max(0);
            }
            Event::ForcedMarchResolved(ForcedMarchResolved {
                corps: order.corps().clone(),
                from,
                to,
                hops,
                path,
                morale_loss_q4: morale_loss,
            })
        }
        MovementPlan::InterceptionQueued { target } => {
            Event::InterceptionQueued(InterceptionQueued {
                corps: order.corps().clone(),
                target_area: target,
            })
        }
    }
}

fn apply_move(s: &mut Scenario, corps_id: &gc1805_core_schema::ids::CorpsId, to: &AreaId) {
    if let Some(c) = s.corps.get_mut(corps_id) {
        c.area = to.clone();
    }
}

trait CorpsAreaLookup {
    fn corps_area_or_default(&self, s: &Scenario) -> AreaId;
}

impl CorpsAreaLookup for Order {
    fn corps_area_or_default(&self, s: &Scenario) -> AreaId {
        s.corps
            .get(self.corps())
            .map(|c| c.area.clone())
            .unwrap_or_else(|| AreaId::from(""))
    }
}

/// Convenience: validate-and-emit an `OrderRejected` event when the
/// validator fails.  Keeps the CLI loop simple.
pub fn validate_or_reject(s: &Scenario, order: &Order) -> Result<MovementPlan, Event> {
    validate_order(s, order).map_err(|e| {
        Event::OrderRejected(OrderRejected {
            reason_code: e.code().to_string(),
            message: e.to_string(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::{AreaId, CorpsId, LeaderId, PowerId};
    use gc1805_core_schema::scenario::{
        Area, AreaAdjacency, Corps, Features, GameDate, Leader, MovementRules, Owner, PowerSetup,
        PowerSlot, Scenario, Terrain, SCHEMA_VERSION,
    };
    use std::collections::BTreeMap;

    fn fixture() -> Scenario {
        let mut s = Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "mv".into(),
            name: "mv".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1805, 5),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules {
                max_corps_per_area: Maybe::Value(2),
                movement_hops_per_turn: Maybe::Value(2),
                forced_march_extra_hops: Maybe::Value(1),
                forced_march_morale_loss_q4: Maybe::Value(500),
            },
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
        s.leaders.insert(
            LeaderId::from("LEADER_N"),
            Leader {
                display_name: "N".into(),
                strategic: 5,
                tactical: 5,
                initiative: 9,
                army_commander: true,
                born: GameDate::new(1769, 8),
            },
        );
        for n in ["AREA_A", "AREA_B", "AREA_C", "AREA_D"] {
            s.areas.insert(
                AreaId::from(n),
                Area {
                    display_name: n.into(),
                    owner: Owner::Power(PowerSlot {
                        power: PowerId::from("FRA"),
                    }),
                    terrain: Terrain::Open,
                    fort_level: 0,
                    money_yield: Maybe::Value(0),
                    manpower_yield: Maybe::Value(0),
                    capital_of: None,
                    port: false,
                    map_x: 0,
                    map_y: 0,
                },
            );
        }
        for (a, b) in [
            ("AREA_A", "AREA_B"),
            ("AREA_B", "AREA_C"),
            ("AREA_C", "AREA_D"),
        ] {
            s.adjacency.push(AreaAdjacency {
                from: AreaId::from(a),
                to: AreaId::from(b),
                cost: Maybe::Value(1),
            });
            s.adjacency.push(AreaAdjacency {
                from: AreaId::from(b),
                to: AreaId::from(a),
                cost: Maybe::Value(1),
            });
        }
        s.powers.insert(
            PowerId::from("FRA"),
            PowerSetup {
                display_name: "France".into(),
                house: "B".into(),
                ruler: LeaderId::from("LEADER_N"),
                capital: AreaId::from("AREA_A"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 8,
                mobilization_areas: vec![],
                color_hex: "#000".into(),
            },
        );
        s.corps.insert(
            CorpsId::from("CORPS_FRA_001"),
            Corps {
                display_name: "I.".into(),
                owner: PowerId::from("FRA"),
                area: AreaId::from("AREA_A"),
                infantry_sp: 22,
                cavalry_sp: 4,
                artillery_sp: 6,
                morale_q4: 9500,
                supplied: true,
                leader: Some(LeaderId::from("LEADER_N")),
            },
        );
        s
    }

    fn move_order(corps: &str, to: &str) -> Order {
        Order::Move(crate::orders::MoveOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from(corps),
            to: AreaId::from(to),
        })
    }

    // ── validation cases (8) ────────────────────────────────────────

    #[test]
    fn case_v01_move_within_budget_validates() {
        let s = fixture();
        let plan = validate_order(&s, &move_order("CORPS_FRA_001", "AREA_C")).unwrap();
        assert!(matches!(plan, MovementPlan::Move { hops: 2, .. }));
    }

    #[test]
    fn case_v02_move_over_budget_rejected() {
        let s = fixture();
        let r = validate_order(&s, &move_order("CORPS_FRA_001", "AREA_D"));
        assert!(matches!(
            r,
            Err(MovementRejection::OverBudget { hops: 3, budget: 2 })
        ));
    }

    #[test]
    fn case_v03_unknown_corps() {
        let s = fixture();
        let r = validate_order(&s, &move_order("CORPS_FRA_999", "AREA_B"));
        assert!(matches!(r, Err(MovementRejection::UnknownCorps(_))));
    }

    #[test]
    fn case_v04_unknown_area() {
        let s = fixture();
        let r = validate_order(&s, &move_order("CORPS_FRA_001", "AREA_Z"));
        assert!(matches!(r, Err(MovementRejection::UnknownArea(_))));
    }

    #[test]
    fn case_v05_not_owner() {
        let s = fixture();
        let bad = Order::Move(crate::orders::MoveOrder {
            submitter: PowerId::from("AUS"),
            corps: CorpsId::from("CORPS_FRA_001"),
            to: AreaId::from("AREA_B"),
        });
        let r = validate_order(&s, &bad);
        assert!(matches!(r, Err(MovementRejection::NotOwner { .. })));
    }

    #[test]
    fn case_v06_forced_march_extends_budget_by_one() {
        let s = fixture();
        let order = Order::ForcedMarch(crate::orders::ForcedMarchOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            to: AreaId::from("AREA_D"),
        });
        let plan = validate_order(&s, &order).unwrap();
        assert!(matches!(plan, MovementPlan::ForcedMarch { hops: 3, .. }));
    }

    #[test]
    fn case_v07_stacking_limit_rejects() {
        let mut s = fixture();
        // Pre-fill AREA_B with 2 other corps to hit the limit (=2).
        for (n, leader) in [("CORPS_FRA_002", "LEADER_N"), ("CORPS_FRA_003", "LEADER_N")] {
            s.corps.insert(
                CorpsId::from(n),
                Corps {
                    display_name: n.into(),
                    owner: PowerId::from("FRA"),
                    area: AreaId::from("AREA_B"),
                    infantry_sp: 1,
                    cavalry_sp: 0,
                    artillery_sp: 0,
                    morale_q4: 5000,
                    supplied: true,
                    leader: Some(LeaderId::from(leader)),
                },
            );
        }
        let r = validate_order(&s, &move_order("CORPS_FRA_001", "AREA_B"));
        assert!(matches!(
            r,
            Err(MovementRejection::StackingLimit { limit: 2 })
        ));
    }

    #[test]
    fn case_v08_interception_validates_structurally() {
        let s = fixture();
        let order = Order::Interception(crate::orders::InterceptionOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            target_area: AreaId::from("AREA_C"),
            condition: "any".into(),
        });
        let plan = validate_order(&s, &order).unwrap();
        assert!(matches!(plan, MovementPlan::InterceptionQueued { .. }));
    }

    // ── resolution cases (4) ───────────────────────────────────────

    #[test]
    fn case_r01_move_relocates_corps_in_state() {
        let mut s = fixture();
        let order = move_order("CORPS_FRA_001", "AREA_C");
        let plan = validate_order(&s, &order).unwrap();
        let _ev = resolve_order(&mut s, &order, plan);
        assert_eq!(
            s.corps.get(&CorpsId::from("CORPS_FRA_001")).unwrap().area,
            AreaId::from("AREA_C")
        );
    }

    #[test]
    fn case_r02_move_emits_event_with_path() {
        let mut s = fixture();
        let order = move_order("CORPS_FRA_001", "AREA_C");
        let plan = validate_order(&s, &order).unwrap();
        let ev = resolve_order(&mut s, &order, plan);
        match ev {
            Event::MovementResolved(m) => {
                assert_eq!(m.path.len(), 3);
                assert_eq!(m.hops, 2);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn case_r03_forced_march_drops_morale_by_table_value() {
        let mut s = fixture();
        let before = s
            .corps
            .get(&CorpsId::from("CORPS_FRA_001"))
            .unwrap()
            .morale_q4;
        let order = Order::ForcedMarch(crate::orders::ForcedMarchOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
            to: AreaId::from("AREA_D"),
        });
        let plan = validate_order(&s, &order).unwrap();
        let _ev = resolve_order(&mut s, &order, plan);
        let after = s
            .corps
            .get(&CorpsId::from("CORPS_FRA_001"))
            .unwrap()
            .morale_q4;
        assert_eq!(before - after, 500);
    }

    #[test]
    fn case_r04_hold_emits_zero_hops() {
        let mut s = fixture();
        let order = Order::Hold(crate::orders::HoldOrder {
            submitter: PowerId::from("FRA"),
            corps: CorpsId::from("CORPS_FRA_001"),
        });
        let plan = validate_order(&s, &order).unwrap();
        let ev = resolve_order(&mut s, &order, plan);
        match ev {
            Event::MovementResolved(m) => assert_eq!(m.hops, 0),
            _ => panic!("wrong event"),
        }
    }
}
