//! Economic phase resolver (PROMPT.md §16.4, `docs/rules/economy.md`).
//!
//! Three public entry points:
//!
//! - [`resolve_economic_phase`] — runs once per turn in the order:
//!   income → maintenance → replacements → production → subsidies.
//! - [`validate_economic_order`] — pure check, never mutates.
//! - [`apply_economic_order`] — accepts a validated order and mutates
//!   the scenario.
//!
//! HARD RULES (PROMPT.md §0):
//! - No floats.
//! - No wall-clock time.
//! - No HashMap in simulation logic.
//! - Designer-authored numerics stay `Maybe::Placeholder` until authored.

use gc1805_core_schema::events::{Event, OrderRejected};
use gc1805_core_schema::ids::{CorpsId, FleetId, PowerId};
use gc1805_core_schema::scenario::{
    Corps, DiplomaticPairKey, DiplomaticState, Fleet, Owner, PendingSubsidy, ProductionItem,
    ProductionKind, ReplacementItem, TaxPolicy,
};
use gc1805_core_schema::tables::{EconomyTable, Maybe};

use crate::orders::{BuildCorpsOrder, BuildFleetOrder, Order, SubsidyOrder};
use gc1805_core_schema::scenario::Scenario;

// ─── Public entry points ───────────────────────────────────────────────

/// Run the full economic phase for all powers in BTreeMap (deterministic)
/// order.  Mutates `scenario` in place and returns the ordered event log.
pub fn resolve_economic_phase(scenario: &mut Scenario, tables: &EconomyTable) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    // ── 1. Income ──────────────────────────────────────────────────────
    // Collect power IDs first to satisfy the borrow checker.
    let power_ids: Vec<PowerId> = scenario.power_state.keys().cloned().collect();
    for power_id in &power_ids {
        let gross = income_gross(scenario, power_id);
        let tax_policy = scenario
            .power_state
            .get(power_id)
            .map(|ps| ps.tax_policy)
            .unwrap_or(TaxPolicy::Standard);
        let multiplier = tax_multiplier(tables, tax_policy);
        let net = (gross * multiplier) / 10_000;

        if let Some(ps) = scenario.power_state.get_mut(power_id) {
            ps.treasury += net;
        }
        events.push(Event::IncomePaid {
            power: power_id.clone(),
            gross,
            net,
            tax_policy,
        });
    }

    // ── 2. Maintenance ─────────────────────────────────────────────────
    for power_id in &power_ids {
        let corps_sp: i64 = scenario
            .corps
            .values()
            .filter(|c| &c.owner == power_id)
            .map(|c| (c.infantry_sp + c.cavalry_sp + c.artillery_sp) as i64)
            .sum();

        let fleet_ships: i64 = scenario
            .fleets
            .values()
            .filter(|f| &f.owner == power_id)
            .map(|f| (f.ships_of_the_line + f.frigates + f.transports) as i64)
            .sum();

        let corps_cost = match &tables.corps_maintenance_per_sp {
            Maybe::Value(rate) => corps_sp * (*rate as i64),
            Maybe::Placeholder(_) => 0,
        };
        let fleet_cost = match &tables.fleet_maintenance_per_ship {
            Maybe::Value(rate) => fleet_ships * (*rate as i64),
            Maybe::Placeholder(_) => 0,
        };

        // Skip event if both rates are placeholder (no deduction).
        let rates_are_placeholder = tables.corps_maintenance_per_sp.is_placeholder()
            && tables.fleet_maintenance_per_ship.is_placeholder();

        let total_cost = corps_cost + fleet_cost;

        if let Some(ps) = scenario.power_state.get_mut(power_id) {
            if total_cost > ps.treasury {
                let shortfall = total_cost - ps.treasury;
                ps.treasury = 0;
                events.push(Event::MaintenancePaid {
                    power: power_id.clone(),
                    corps_cost,
                    fleet_cost,
                });
                events.push(Event::TreasuryInDeficit {
                    power: power_id.clone(),
                    shortfall,
                });
            } else {
                ps.treasury -= total_cost;
                if !rates_are_placeholder {
                    events.push(Event::MaintenancePaid {
                        power: power_id.clone(),
                        corps_cost,
                        fleet_cost,
                    });
                }
            }
        }
    }

    // ── 3. Replacements ────────────────────────────────────────────────
    let current_turn = scenario.current_turn;
    let mut remaining_replacements: Vec<ReplacementItem> = Vec::new();
    for item in scenario.replacement_queue.drain(..) {
        if item.eta_turn == current_turn {
            if let Some(ps) = scenario.power_state.get_mut(&item.owner) {
                ps.manpower += item.sp_amount;
            }
            events.push(Event::ReplacementsArrived {
                owner: item.owner,
                sp_amount: item.sp_amount,
            });
        } else {
            remaining_replacements.push(item);
        }
    }
    scenario.replacement_queue = remaining_replacements;

    // ── 4. Production ──────────────────────────────────────────────────
    let mut remaining_production: Vec<ProductionItem> = Vec::new();
    let mut spawned: Vec<ProductionItem> = Vec::new();
    for item in scenario.production_queue.drain(..) {
        if item.eta_turn == current_turn {
            spawned.push(item);
        } else {
            remaining_production.push(item);
        }
    }
    scenario.production_queue = remaining_production;

    for item in spawned {
        match item.kind {
            ProductionKind::Corps => {
                let morale = match &tables.new_corps_morale_q4 {
                    Maybe::Value(v) => *v,
                    Maybe::Placeholder(_) => 0,
                };
                let composition = item.corps_composition.clone().unwrap_or(
                    gc1805_core_schema::scenario::CorpsComposition {
                        infantry_sp: 4,
                        cavalry_sp: 1,
                        artillery_sp: 1,
                    },
                );
                // Deterministic ID: CORPS_<POWER>_<turn>_<area>
                let id_str = format!(
                    "CORPS_{}_T{}_{}",
                    item.owner,
                    current_turn,
                    item.area.as_str().replace("AREA_", "")
                );
                let corps_id = CorpsId::from(id_str.as_str());
                scenario.corps.insert(
                    corps_id,
                    Corps {
                        display_name: format!("New Corps ({})", item.owner),
                        owner: item.owner.clone(),
                        area: item.area.clone(),
                        infantry_sp: composition.infantry_sp,
                        cavalry_sp: composition.cavalry_sp,
                        artillery_sp: composition.artillery_sp,
                        morale_q4: morale,
                        supplied: true,
                        leader: None,
                    },
                );
                events.push(Event::UnitProduced {
                    owner: item.owner,
                    area: item.area,
                    unit_kind: ProductionKind::Corps,
                });
            }
            ProductionKind::Fleet => {
                let id_str = format!(
                    "FLEET_{}_T{}_{}",
                    item.owner,
                    current_turn,
                    item.area.as_str().replace("AREA_", "")
                );
                let fleet_id = FleetId::from(id_str.as_str());
                scenario.fleets.insert(
                    fleet_id,
                    Fleet {
                        display_name: format!("New Fleet ({})", item.owner),
                        owner: item.owner.clone(),
                        at_port: Some(item.area.clone()),
                        at_sea: None,
                        ships_of_the_line: 1,
                        frigates: 0,
                        transports: 0,
                        morale_q4: 10_000,
                        admiral: None,
                        embarked_corps: Vec::new(),
                    },
                );
                events.push(Event::UnitProduced {
                    owner: item.owner,
                    area: item.area,
                    unit_kind: ProductionKind::Fleet,
                });
            }
            ProductionKind::Depot => {
                // Depot spawning is Phase 5; skip silently for now.
            }
        }
    }

    // ── 5. Subsidies ───────────────────────────────────────────────────
    let subsidies: Vec<PendingSubsidy> = scenario.subsidy_queue.drain(..).collect();
    for sub in subsidies {
        // Transfer: deduct from sender, add to recipient.
        // If sender can't pay, clamp (no hard block here — validation
        // should have caught this, but be defensive).
        let actual_amount = if let Some(from_ps) = scenario.power_state.get(&sub.from) {
            sub.amount.min(from_ps.treasury)
        } else {
            0
        };
        if actual_amount > 0 {
            if let Some(from_ps) = scenario.power_state.get_mut(&sub.from) {
                from_ps.treasury -= actual_amount;
            }
            if let Some(to_ps) = scenario.power_state.get_mut(&sub.to) {
                to_ps.treasury += actual_amount;
            }
        }
        events.push(Event::SubsidyTransferred {
            from: sub.from,
            to: sub.to,
            amount: actual_amount,
        });
    }

    events
}

// ─── Validation ────────────────────────────────────────────────────────

/// Pure validation; never mutates.  Returns `Ok(())` or a descriptive
/// error string that callers may wrap in `OrderRejected`.
pub fn validate_economic_order(
    scenario: &Scenario,
    tables: &EconomyTable,
    order: &Order,
) -> Result<(), String> {
    match order {
        Order::SetTaxPolicy(_) => Ok(()),

        Order::BuildCorps(o) => validate_build_corps(scenario, tables, o),

        Order::BuildFleet(o) => validate_build_fleet(scenario, tables, o),

        Order::Subsidize(o) => validate_subsidize(scenario, o),

        _ => Err(format!(
            "order kind `{}` is not an economic order",
            order_kind_name(order)
        )),
    }
}

/// Apply a validated economic order.  Mutates `scenario` and returns the
/// resulting event.  Callers should call `validate_economic_order` first.
pub fn apply_economic_order(
    scenario: &mut Scenario,
    tables: &EconomyTable,
    order: &Order,
) -> Event {
    match order {
        Order::SetTaxPolicy(o) => {
            if let Some(ps) = scenario.power_state.get_mut(&o.submitter) {
                ps.tax_policy = o.policy;
            }
            Event::TaxPolicySet {
                power: o.submitter.clone(),
                new_policy: o.policy,
            }
        }

        Order::BuildCorps(o) => {
            // Deduct costs if values are known.
            if let Maybe::Value(cost_money) = &tables.corps_build_cost_money {
                if let Some(ps) = scenario.power_state.get_mut(&o.submitter) {
                    ps.treasury -= *cost_money as i64;
                }
            }
            if let Maybe::Value(cost_mp) = &tables.corps_build_cost_manpower {
                if let Some(ps) = scenario.power_state.get_mut(&o.submitter) {
                    ps.manpower -= *cost_mp;
                }
            }
            let eta = scenario.current_turn
                + match &tables.corps_production_lag_turns {
                    Maybe::Value(lag) => *lag as u32,
                    Maybe::Placeholder(_) => 0,
                };
            scenario.production_queue.push(ProductionItem {
                owner: o.submitter.clone(),
                area: o.area.clone(),
                kind: ProductionKind::Corps,
                eta_turn: eta,
                corps_composition: Some(o.composition.clone()),
            });
            Event::OrderRejected(OrderRejected {
                reason_code: "BUILD_CORPS_QUEUED".into(),
                message: format!(
                    "Corps build queued for {} at {}, ETA turn {}",
                    o.submitter, o.area, eta
                ),
            })
        }

        Order::BuildFleet(o) => {
            if let Maybe::Value(cost_money) = &tables.fleet_build_cost_money {
                if let Some(ps) = scenario.power_state.get_mut(&o.submitter) {
                    ps.treasury -= *cost_money as i64;
                }
            }
            let eta = scenario.current_turn
                + match &tables.fleet_production_lag_turns {
                    Maybe::Value(lag) => *lag as u32,
                    Maybe::Placeholder(_) => 0,
                };
            scenario.production_queue.push(ProductionItem {
                owner: o.submitter.clone(),
                area: o.area.clone(),
                kind: ProductionKind::Fleet,
                eta_turn: eta,
                corps_composition: None,
            });
            Event::OrderRejected(OrderRejected {
                reason_code: "BUILD_FLEET_QUEUED".into(),
                message: format!(
                    "Fleet build queued for {} at {}, ETA turn {}",
                    o.submitter, o.area, eta
                ),
            })
        }

        Order::Subsidize(o) => {
            scenario.subsidy_queue.push(PendingSubsidy {
                from: o.submitter.clone(),
                to: o.recipient.clone(),
                amount: o.amount,
            });
            Event::OrderRejected(OrderRejected {
                reason_code: "SUBSIDY_QUEUED".into(),
                message: format!(
                    "Subsidy of {} from {} to {} queued for next economic phase",
                    o.amount, o.submitter, o.recipient
                ),
            })
        }

        // Non-economic orders — return rejection.
        other => Event::OrderRejected(OrderRejected {
            reason_code: "NOT_ECONOMIC_ORDER".into(),
            message: format!(
                "order `{}` is not handled by the economic phase",
                order_kind_name(other)
            ),
        }),
    }
}

// ─── Internal helpers ──────────────────────────────────────────────────

/// Sum the money yield of all non-blockaded areas owned by `power`.
fn income_gross(scenario: &Scenario, power: &PowerId) -> i64 {
    scenario
        .areas
        .values()
        .filter(|a| matches!(&a.owner, Owner::Power(slot) if &slot.power == power) && !a.blockaded)
        .filter_map(|a| a.money_yield.get().copied())
        .map(|y| y as i64)
        .sum()
}

/// Return the Q4 tax multiplier for the given policy.  Falls back to
/// identity (10 000) if the designer hasn't filled in the table yet.
fn tax_multiplier(tables: &EconomyTable, policy: TaxPolicy) -> i64 {
    let maybe = match policy {
        TaxPolicy::Low => &tables.tax_policy_multiplier_low_q4,
        TaxPolicy::Standard => &tables.tax_policy_multiplier_standard_q4,
        TaxPolicy::Heavy => &tables.tax_policy_multiplier_heavy_q4,
    };
    match maybe {
        Maybe::Value(v) => *v as i64,
        Maybe::Placeholder(_) => 10_000, // identity: net == gross
    }
}

fn validate_build_corps(
    scenario: &Scenario,
    tables: &EconomyTable,
    o: &BuildCorpsOrder,
) -> Result<(), String> {
    // Area must be the power's capital or a declared mobilization area.
    let power_setup = scenario
        .powers
        .get(&o.submitter)
        .ok_or_else(|| format!("unknown power `{}`", o.submitter))?;

    let is_capital = power_setup.capital == o.area;
    let is_mob_area = power_setup.mobilization_areas.contains(&o.area);
    if !is_capital && !is_mob_area {
        return Err(format!(
            "area `{}` is not the capital or a mobilization area of `{}`",
            o.area, o.submitter
        ));
    }

    // Check treasury if value is known.
    if let Maybe::Value(cost_money) = &tables.corps_build_cost_money {
        let treasury = scenario
            .power_state
            .get(&o.submitter)
            .map(|ps| ps.treasury)
            .unwrap_or(0);
        if treasury < *cost_money as i64 {
            return Err(format!(
                "`{}` cannot afford corps build: treasury={} cost={}",
                o.submitter, treasury, cost_money
            ));
        }
    }

    // Check manpower if value is known.
    if let Maybe::Value(cost_mp) = &tables.corps_build_cost_manpower {
        let manpower = scenario
            .power_state
            .get(&o.submitter)
            .map(|ps| ps.manpower)
            .unwrap_or(0);
        if manpower < *cost_mp {
            return Err(format!(
                "`{}` cannot afford corps build: manpower={} required={}",
                o.submitter, manpower, cost_mp
            ));
        }
    }

    Ok(())
}

fn validate_build_fleet(
    scenario: &Scenario,
    tables: &EconomyTable,
    o: &BuildFleetOrder,
) -> Result<(), String> {
    let power_setup = scenario
        .powers
        .get(&o.submitter)
        .ok_or_else(|| format!("unknown power `{}`", o.submitter))?;

    let is_capital = power_setup.capital == o.area;
    let is_mob_area = power_setup.mobilization_areas.contains(&o.area);
    if !is_capital && !is_mob_area {
        return Err(format!(
            "area `{}` is not the capital or a mobilization area of `{}`",
            o.area, o.submitter
        ));
    }

    if let Maybe::Value(cost_money) = &tables.fleet_build_cost_money {
        let treasury = scenario
            .power_state
            .get(&o.submitter)
            .map(|ps| ps.treasury)
            .unwrap_or(0);
        if treasury < *cost_money as i64 {
            return Err(format!(
                "`{}` cannot afford fleet build: treasury={} cost={}",
                o.submitter, treasury, cost_money
            ));
        }
    }

    Ok(())
}

fn validate_subsidize(scenario: &Scenario, o: &SubsidyOrder) -> Result<(), String> {
    if o.amount <= 0 {
        return Err(format!("subsidy amount must be positive, got {}", o.amount));
    }
    if o.submitter == o.recipient {
        return Err("cannot subsidize yourself".into());
    }

    // Check sender treasury if known.
    let treasury = scenario
        .power_state
        .get(&o.submitter)
        .map(|ps| ps.treasury)
        .unwrap_or(0);
    if treasury < o.amount {
        return Err(format!(
            "`{}` cannot afford subsidy: treasury={} amount={}",
            o.submitter, treasury, o.amount
        ));
    }

    // Powers at war cannot exchange subsidies.
    let key = DiplomaticPairKey::new(o.submitter.clone(), o.recipient.clone());
    if let Some(DiplomaticState::War) = scenario.diplomacy.get(&key) {
        return Err(format!(
            "`{}` and `{}` are at war; subsidies forbidden",
            o.submitter, o.recipient
        ));
    }

    Ok(())
}

fn order_kind_name(order: &Order) -> &'static str {
    match order {
        Order::Hold(_) => "Hold",
        Order::Move(_) => "Move",
        Order::ForcedMarch(_) => "ForcedMarch",
        Order::Interception(_) => "Interception",
        Order::SetTaxPolicy(_) => "SetTaxPolicy",
        Order::BuildCorps(_) => "BuildCorps",
        Order::BuildFleet(_) => "BuildFleet",
        Order::Subsidize(_) => "Subsidize",
        Order::Attack(_) => "Attack",
        Order::Bombard(_) => "Bombard",
        Order::MoveFleet(_) => "MoveFleet",
        Order::NavalAttack(_) => "NavalAttack",
        Order::Embark(_) => "Embark",
        Order::Disembark(_) => "Disembark",
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::{AreaId, CorpsId};
    use gc1805_core_schema::scenario::{
        Area, Corps, GameDate, Owner, PowerSetup, PowerSlot, PowerState, Scenario, TaxPolicy,
    };
    use gc1805_core_schema::tables::{EconomyTable, Maybe};
    use std::collections::BTreeMap;

    // ── Fixtures ─────────────────────────────────────────────────────

    fn fra() -> PowerId {
        PowerId::from("FRA")
    }
    fn gbr() -> PowerId {
        PowerId::from("GBR")
    }
    fn area_paris() -> AreaId {
        AreaId::from("AREA_PARIS")
    }

    fn corps_fra_001() -> CorpsId {
        CorpsId::from("CORPS_FRA_001")
    }

    fn minimal_scenario() -> Scenario {
        let mut powers = BTreeMap::new();
        powers.insert(
            fra(),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: gc1805_core_schema::ids::LeaderId::from("LEADER_NAPOLEON"),
                capital: area_paris(),
                starting_treasury: 100,
                starting_manpower: 50,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 3,
                mobilization_areas: vec![],
                color_hex: "#2a3a6a".into(),
            },
        );

        let mut areas = BTreeMap::new();
        areas.insert(
            area_paris(),
            Area {
                display_name: "Paris".into(),
                owner: Owner::Power(PowerSlot { power: fra() }),
                terrain: gc1805_core_schema::scenario::Terrain::Urban,
                fort_level: 2,
                money_yield: Maybe::Value(10),
                manpower_yield: Maybe::Placeholder(Default::default()),
                capital_of: Some(fra()),
                port: false,
                blockaded: false,
                map_x: 0,
                map_y: 0,
            },
        );

        let mut corps = BTreeMap::new();
        corps.insert(
            corps_fra_001(),
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

        Scenario {
            schema_version: 1,
            rules_version: 0,
            scenario_id: "test".into(),
            name: "Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: true,
            features: Default::default(),
            movement_rules: Default::default(),
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
            diplomacy: BTreeMap::new(),
            adjacency: vec![],
            coast_links: vec![],
            sea_adjacency: vec![],
        }
    }

    fn standard_tables() -> EconomyTable {
        EconomyTable {
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
        }
    }

    // ── Income tests ─────────────────────────────────────────────────

    /// 1. income_basic: yield=10, standard multiplier → treasury 100→110
    #[test]
    fn income_basic() {
        let mut s = minimal_scenario();
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        assert_eq!(s.power_state[&fra()].treasury, 110 - 12); // income 10, maintenance 6sp*2=12 → 98
                                                              // But let's check the IncomePaid event specifically
        let income_ev = events
            .iter()
            .find(|e| matches!(e, Event::IncomePaid { .. }));
        assert!(income_ev.is_some());
        if let Some(Event::IncomePaid { gross, net, .. }) = income_ev {
            assert_eq!(*gross, 10);
            assert_eq!(*net, 10); // 10 * 10000 / 10000
        }
    }

    /// 2. income_low_tax: Low policy → net=8
    #[test]
    fn income_low_tax() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().tax_policy = TaxPolicy::Low;
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        let income_ev = events
            .iter()
            .find(|e| matches!(e, Event::IncomePaid { .. }));
        if let Some(Event::IncomePaid {
            gross,
            net,
            tax_policy,
            ..
        }) = income_ev
        {
            assert_eq!(*gross, 10);
            assert_eq!(*net, 8); // 10 * 8000 / 10000 = 8
            assert_eq!(*tax_policy, TaxPolicy::Low);
        } else {
            panic!("no IncomePaid event");
        }
    }

    /// 3. income_heavy_tax: Heavy policy → net=12
    #[test]
    fn income_heavy_tax() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().tax_policy = TaxPolicy::Heavy;
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        let income_ev = events
            .iter()
            .find(|e| matches!(e, Event::IncomePaid { .. }));
        if let Some(Event::IncomePaid {
            gross,
            net,
            tax_policy,
            ..
        }) = income_ev
        {
            assert_eq!(*gross, 10);
            assert_eq!(*net, 12);
            assert_eq!(*tax_policy, TaxPolicy::Heavy);
        } else {
            panic!("no IncomePaid event");
        }
    }

    /// 4. income_blockaded_excluded: blockaded=true → net=0
    #[test]
    fn income_blockaded_excluded() {
        let mut s = minimal_scenario();
        s.areas.get_mut(&area_paris()).unwrap().blockaded = true;
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        let income_ev = events
            .iter()
            .find(|e| matches!(e, Event::IncomePaid { .. }));
        if let Some(Event::IncomePaid { gross, net, .. }) = income_ev {
            assert_eq!(*gross, 0);
            assert_eq!(*net, 0);
        } else {
            panic!("no IncomePaid event");
        }
    }

    /// 5. income_placeholder_multiplier: all multipliers Placeholder → net==gross
    #[test]
    fn income_placeholder_multiplier() {
        let mut s = minimal_scenario();
        let mut t = standard_tables();
        t.tax_policy_multiplier_low_q4 = Maybe::Placeholder(Default::default());
        t.tax_policy_multiplier_standard_q4 = Maybe::Placeholder(Default::default());
        t.tax_policy_multiplier_heavy_q4 = Maybe::Placeholder(Default::default());
        let events = resolve_economic_phase(&mut s, &t);
        let income_ev = events
            .iter()
            .find(|e| matches!(e, Event::IncomePaid { .. }));
        if let Some(Event::IncomePaid { gross, net, .. }) = income_ev {
            assert_eq!(gross, net); // identity: net == gross == 10
            assert_eq!(*gross, 10);
        } else {
            panic!("no IncomePaid event");
        }
    }

    /// 6. income_placeholder_yield: yield Placeholder → net=0
    #[test]
    fn income_placeholder_yield() {
        let mut s = minimal_scenario();
        s.areas.get_mut(&area_paris()).unwrap().money_yield =
            Maybe::Placeholder(Default::default());
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        let income_ev = events
            .iter()
            .find(|e| matches!(e, Event::IncomePaid { .. }));
        if let Some(Event::IncomePaid { gross, net, .. }) = income_ev {
            assert_eq!(*gross, 0);
            assert_eq!(*net, 0);
        } else {
            panic!("no IncomePaid event");
        }
    }

    // ── Maintenance tests ────────────────────────────────────────────

    /// 7. maintenance_corps_deducted: 6sp * 2 = 12 deducted.
    ///
    /// Starting=100, income net=10 (std mult), maintenance=(4+1+1)*2=12 → treasury=98.
    #[test]
    fn maintenance_corps_deducted() {
        let mut s = minimal_scenario();
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        // income: 10 (standard), maintenance: (4+1+1)*2=12
        // 100 + 10 - 12 = 98
        assert_eq!(s.power_state[&fra()].treasury, 98);
        let maint_ev = events
            .iter()
            .find(|e| matches!(e, Event::MaintenancePaid { .. }));
        if let Some(Event::MaintenancePaid {
            corps_cost,
            fleet_cost,
            ..
        }) = maint_ev
        {
            assert_eq!(*corps_cost, 12);
            assert_eq!(*fleet_cost, 0);
        } else {
            panic!("no MaintenancePaid event");
        }
    }

    /// 8. maintenance_deficit_clamped: treasury=5, corps cost=12 → treasury=0, TreasuryInDeficit
    #[test]
    fn maintenance_deficit_clamped() {
        let mut s = minimal_scenario();
        // Zero income (blockaded) to isolate
        s.areas.get_mut(&area_paris()).unwrap().blockaded = true;
        s.power_state.get_mut(&fra()).unwrap().treasury = 5;
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        // income=0, maintenance=12 but treasury=5 → clamp to 0, deficit=12
        assert_eq!(s.power_state[&fra()].treasury, 0);
        let deficit_ev = events
            .iter()
            .find(|e| matches!(e, Event::TreasuryInDeficit { .. }));
        assert!(deficit_ev.is_some(), "expected TreasuryInDeficit event");
        if let Some(Event::TreasuryInDeficit { shortfall, .. }) = deficit_ev {
            assert_eq!(*shortfall, 7); // treasury=5, cost=12, shortfall=12-5=7
        }
    }

    /// 9. maintenance_placeholder_skipped: rates Placeholder → no MaintenancePaid
    #[test]
    fn maintenance_placeholder_skipped() {
        let mut s = minimal_scenario();
        let mut t = standard_tables();
        t.corps_maintenance_per_sp = Maybe::Placeholder(Default::default());
        t.fleet_maintenance_per_ship = Maybe::Placeholder(Default::default());
        let events = resolve_economic_phase(&mut s, &t);
        let maint_ev = events
            .iter()
            .find(|e| matches!(e, Event::MaintenancePaid { .. }));
        assert!(
            maint_ev.is_none(),
            "should not emit MaintenancePaid with placeholder rates"
        );
        // treasury after: 100 + 10 income = 110
        assert_eq!(s.power_state[&fra()].treasury, 110);
    }

    // ── Replacement tests ────────────────────────────────────────────

    /// 10. replacements_arrive: item eta_turn=0, manpower 50→55
    #[test]
    fn replacements_arrive() {
        let mut s = minimal_scenario();
        s.replacement_queue.push(ReplacementItem {
            owner: fra(),
            sp_amount: 5,
            eta_turn: 0,
        });
        let t = standard_tables();
        resolve_economic_phase(&mut s, &t);
        assert_eq!(s.power_state[&fra()].manpower, 55);
        assert!(s.replacement_queue.is_empty());
    }

    /// 11. replacements_future_stays: eta_turn=1, queue unchanged
    #[test]
    fn replacements_future_stays() {
        let mut s = minimal_scenario();
        s.replacement_queue.push(ReplacementItem {
            owner: fra(),
            sp_amount: 5,
            eta_turn: 1,
        });
        let t = standard_tables();
        resolve_economic_phase(&mut s, &t);
        assert_eq!(s.power_state[&fra()].manpower, 50); // unchanged
        assert_eq!(s.replacement_queue.len(), 1);
    }

    /// 12. replacements_multiple: two items eta_turn=0, both processed
    #[test]
    fn replacements_multiple() {
        let mut s = minimal_scenario();
        s.replacement_queue.push(ReplacementItem {
            owner: fra(),
            sp_amount: 3,
            eta_turn: 0,
        });
        s.replacement_queue.push(ReplacementItem {
            owner: fra(),
            sp_amount: 7,
            eta_turn: 0,
        });
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        assert_eq!(s.power_state[&fra()].manpower, 60);
        let arrivals: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, Event::ReplacementsArrived { .. }))
            .collect();
        assert_eq!(arrivals.len(), 2);
    }

    // ── Production tests ─────────────────────────────────────────────

    /// 13. production_corps_spawned: item kind=Corps eta_turn=0 → corps.len() increases
    #[test]
    fn production_corps_spawned() {
        let mut s = minimal_scenario();
        s.production_queue.push(ProductionItem {
            owner: fra(),
            area: area_paris(),
            kind: ProductionKind::Corps,
            eta_turn: 0,
            corps_composition: None,
        });
        let initial_count = s.corps.len();
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        assert_eq!(s.corps.len(), initial_count + 1);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::UnitProduced {
                unit_kind: ProductionKind::Corps,
                ..
            }
        )));
    }

    /// 14. production_fleet_spawned: kind=Fleet eta_turn=0 → fleets.len() increases
    #[test]
    fn production_fleet_spawned() {
        let mut s = minimal_scenario();
        s.production_queue.push(ProductionItem {
            owner: fra(),
            area: area_paris(),
            kind: ProductionKind::Fleet,
            eta_turn: 0,
            corps_composition: None,
        });
        let initial_count = s.fleets.len();
        let t = standard_tables();
        let events = resolve_economic_phase(&mut s, &t);
        assert_eq!(s.fleets.len(), initial_count + 1);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::UnitProduced {
                unit_kind: ProductionKind::Fleet,
                ..
            }
        )));
    }

    /// 15. production_future_stays: eta_turn=5, queue unchanged
    #[test]
    fn production_future_stays() {
        let mut s = minimal_scenario();
        s.production_queue.push(ProductionItem {
            owner: fra(),
            area: area_paris(),
            kind: ProductionKind::Corps,
            eta_turn: 5,
            corps_composition: None,
        });
        let t = standard_tables();
        resolve_economic_phase(&mut s, &t);
        assert_eq!(s.production_queue.len(), 1);
    }

    // ── Subsidy tests ────────────────────────────────────────────────

    /// 16. subsidy_transfer: PendingSubsidy FRA→GBR amt=20
    #[test]
    fn subsidy_transfer() {
        let mut s = minimal_scenario();
        // Add GBR
        s.power_state.insert(
            gbr(),
            PowerState {
                treasury: 50,
                manpower: 30,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        // Zero income for GBR (no areas), so we isolate the subsidy effect
        s.subsidy_queue.push(PendingSubsidy {
            from: fra(),
            to: gbr(),
            amount: 20,
        });
        // To isolate FRA's subsidy change, use blockaded + placeholder maintenance
        let mut t = standard_tables();
        t.corps_maintenance_per_sp = Maybe::Placeholder(Default::default());
        t.fleet_maintenance_per_ship = Maybe::Placeholder(Default::default());
        t.tax_policy_multiplier_standard_q4 = Maybe::Placeholder(Default::default());
        // Now: FRA income=0 (placeholder mult → identity, gross=10, net=10 actually)
        // Hmm, let me just check relative changes:
        let fra_before = s.power_state[&fra()].treasury;
        let gbr_before = s.power_state[&gbr()].treasury;
        resolve_economic_phase(&mut s, &t);
        // FRA gets +10 income, -20 subsidy
        let fra_after = s.power_state[&fra()].treasury;
        let gbr_after = s.power_state[&gbr()].treasury;
        // The subsidy was 20
        assert_eq!(fra_after, fra_before + 10 - 20); // income 10 (placeholder mult = identity), minus 20 subsidy
        assert_eq!(gbr_after, gbr_before + 20); // GBR: no areas owned, +20 subsidy
        assert!(s.subsidy_queue.is_empty());
    }

    // ── apply_economic_order tests ───────────────────────────────────

    /// 17. tax_policy_change: SetTaxPolicy Heavy
    #[test]
    fn tax_policy_change() {
        let mut s = minimal_scenario();
        let t = standard_tables();
        let order = Order::SetTaxPolicy(crate::orders::SetTaxPolicyOrder {
            submitter: fra(),
            policy: TaxPolicy::Heavy,
        });
        apply_economic_order(&mut s, &t, &order);
        assert_eq!(s.power_state[&fra()].tax_policy, TaxPolicy::Heavy);
    }

    // ── validate_economic_order tests ────────────────────────────────

    /// 18. build_corps_validate_ok: at capital, treasury=100, manpower=50: Ok
    #[test]
    fn build_corps_validate_ok() {
        let s = minimal_scenario();
        let t = standard_tables();
        let order = Order::BuildCorps(crate::orders::BuildCorpsOrder {
            submitter: fra(),
            area: area_paris(), // capital
            composition: gc1805_core_schema::scenario::CorpsComposition {
                infantry_sp: 4,
                cavalry_sp: 1,
                artillery_sp: 1,
            },
        });
        assert!(validate_economic_order(&s, &t, &order).is_ok());
    }

    /// 19. build_corps_validate_broke: treasury=10, cost=50: Err
    #[test]
    fn build_corps_validate_broke() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().treasury = 10;
        let t = standard_tables();
        let order = Order::BuildCorps(crate::orders::BuildCorpsOrder {
            submitter: fra(),
            area: area_paris(),
            composition: gc1805_core_schema::scenario::CorpsComposition {
                infantry_sp: 4,
                cavalry_sp: 1,
                artillery_sp: 1,
            },
        });
        assert!(validate_economic_order(&s, &t, &order).is_err());
    }

    /// 20. subsidy_validate_rejected_war: FRA-GBR at WAR → Err
    #[test]
    fn subsidy_validate_rejected_war() {
        let mut s = minimal_scenario();
        s.power_state.insert(
            gbr(),
            PowerState {
                treasury: 50,
                manpower: 30,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        s.diplomacy
            .insert(DiplomaticPairKey::new(fra(), gbr()), DiplomaticState::War);
        let t = standard_tables();
        let order = Order::Subsidize(crate::orders::SubsidyOrder {
            submitter: fra(),
            recipient: gbr(),
            amount: 20,
        });
        assert!(validate_economic_order(&s, &t, &order).is_err());
    }

    /// 21. subsidy_validate_rejected_broke: treasury=5, amount=100 → Err
    #[test]
    fn subsidy_validate_rejected_broke() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().treasury = 5;
        s.power_state.insert(
            gbr(),
            PowerState {
                treasury: 50,
                manpower: 30,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        let t = standard_tables();
        let order = Order::Subsidize(crate::orders::SubsidyOrder {
            submitter: fra(),
            recipient: gbr(),
            amount: 100,
        });
        assert!(validate_economic_order(&s, &t, &order).is_err());
    }

    /// 22. determinism: clone scenario, run resolve_economic_phase on both → equal events
    #[test]
    fn determinism() {
        let mut s1 = minimal_scenario();
        let mut s2 = s1.clone();
        let t = standard_tables();
        let events1 = resolve_economic_phase(&mut s1, &t);
        let events2 = resolve_economic_phase(&mut s2, &t);
        assert_eq!(events1.len(), events2.len());
        for (e1, e2) in events1.iter().zip(events2.iter()) {
            // Compare serialized JSON for PartialEq across enum variants
            let j1 = serde_json::to_string(e1).unwrap();
            let j2 = serde_json::to_string(e2).unwrap();
            assert_eq!(j1, j2);
        }
        // Also check final treasury state
        assert_eq!(
            s1.power_state[&fra()].treasury,
            s2.power_state[&fra()].treasury
        );
    }
}
