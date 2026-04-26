//! Front line visualization and simplified battle resolution.
//!
//! Models contested areas between warring powers, tracks attack orders,
//! and resolves battles with integer-only arithmetic.  No floats, no
//! wall-clock, BTreeMap/BTreeSet only (PROMPT.md §2).

use gc1805_core_schema::ids::{AreaId, CorpsId, FrontLineId, PowerId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// ─── Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontLine {
    pub id: FrontLineId,
    pub attacker: PowerId,
    pub defender: PowerId,
    /// Areas currently being fought over.
    pub contested_areas: BTreeSet<AreaId>,
    /// Per-area pressure: -100 (defender winning) to +100 (attacker winning).
    pub pressure: BTreeMap<AreaId, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttackOrder {
    pub from_area: AreaId,
    pub to_area: AreaId,
    pub corps_id: CorpsId,
    pub attacker: PowerId,
    pub defender: PowerId,
    pub attacker_strength: u32,
    pub defender_strength: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BattleResult {
    AttackerAdvances,
    Stalemate,
    DefenderHolds,
    DefenderRoutes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleEvent {
    pub area: AreaId,
    pub attacker: PowerId,
    pub defender: PowerId,
    pub attacker_strength: u32,
    pub defender_strength: u32,
    pub result: BattleResult,
    pub casualties_attacker: u32,
    pub casualties_defender: u32,
}

// ─── Front Line Manager ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontLineManager {
    front_lines: BTreeMap<FrontLineId, FrontLine>,
    attack_orders: Vec<AttackOrder>,
    recent_battles: Vec<BattleEvent>,
    next_id: u32,
}

impl FrontLineManager {
    pub fn new() -> Self {
        Self {
            front_lines: BTreeMap::new(),
            attack_orders: Vec::new(),
            recent_battles: Vec::new(),
            next_id: 1,
        }
    }

    /// Issue an attack order from one area to another.
    pub fn issue_attack_order(
        &mut self,
        from_area: AreaId,
        to_area: AreaId,
        corps_id: CorpsId,
        attacker: PowerId,
        defender: PowerId,
        attacker_strength: u32,
        defender_strength: u32,
    ) {
        let order = AttackOrder {
            from_area,
            to_area: to_area.clone(),
            corps_id,
            attacker: attacker.clone(),
            defender: defender.clone(),
            attacker_strength,
            defender_strength,
        };
        self.attack_orders.push(order);

        // Ensure a front line exists for this attacker-defender pair.
        let fl_id = self.find_or_create_front_line(&attacker, &defender);
        if let Some(fl) = self.front_lines.get_mut(&fl_id) {
            fl.contested_areas.insert(to_area.clone());
            // Initial pressure: 0 (even).
            fl.pressure.entry(to_area).or_insert(0);
        }
    }

    fn find_or_create_front_line(
        &mut self,
        attacker: &PowerId,
        defender: &PowerId,
    ) -> FrontLineId {
        // Check if front line already exists for this pair.
        for (id, fl) in &self.front_lines {
            if &fl.attacker == attacker && &fl.defender == defender {
                return id.clone();
            }
        }
        // Create new.
        let id = FrontLineId::from(format!("FRONT_{}_{}", attacker.as_str(), defender.as_str()));
        let fl = FrontLine {
            id: id.clone(),
            attacker: attacker.clone(),
            defender: defender.clone(),
            contested_areas: BTreeSet::new(),
            pressure: BTreeMap::new(),
        };
        self.front_lines.insert(id.clone(), fl);
        id
    }

    /// Resolve all pending attack orders for one tick.
    /// Returns battle events for all resolved combats.
    pub fn resolve_battles(&mut self, tick: u64) -> Vec<BattleEvent> {
        let orders = std::mem::take(&mut self.attack_orders);
        let mut events = Vec::new();

        for order in &orders {
            let event = resolve_single_battle(order, tick);

            // Update pressure on the front line.
            for fl in self.front_lines.values_mut() {
                if fl.attacker == event.attacker && fl.defender == event.defender {
                    let pressure_delta = match event.result {
                        BattleResult::AttackerAdvances => 30,
                        BattleResult::Stalemate => 5,
                        BattleResult::DefenderHolds => -20,
                        BattleResult::DefenderRoutes => 50,
                    };
                    let p = fl.pressure.entry(event.area.clone()).or_insert(0);
                    *p = (*p + pressure_delta).clamp(-100, 100);

                    // If attacker advances, remove from contested.
                    if event.result == BattleResult::AttackerAdvances {
                        fl.contested_areas.remove(&event.area);
                    }
                    // If defender routes, remove from contested.
                    if event.result == BattleResult::DefenderRoutes {
                        fl.contested_areas.remove(&event.area);
                    }
                }
            }

            events.push(event);
        }

        self.recent_battles = events.clone();
        events
    }

    /// Get all current front lines as JSON string.
    pub fn get_front_lines_json(&self) -> String {
        serde_json::to_string(&self.front_lines.values().collect::<Vec<_>>()).unwrap_or_default()
    }

    /// Get recent battle events as JSON string.
    pub fn get_battle_events_json(&self) -> String {
        serde_json::to_string(&self.recent_battles).unwrap_or_default()
    }

    /// Get all active attack orders (pending resolution).
    pub fn get_attack_orders(&self) -> &[AttackOrder] {
        &self.attack_orders
    }

    /// Get all front lines.
    pub fn get_front_lines(&self) -> &BTreeMap<FrontLineId, FrontLine> {
        &self.front_lines
    }

    /// Get recent battle events.
    pub fn get_recent_battles(&self) -> &[BattleEvent] {
        &self.recent_battles
    }
}

// ─── Combat Resolution (integer only) ─────────────────────────────────

/// Simple deterministic RNG: returns 0..9 from a seed.
fn random_d10(seed: u64) -> u32 {
    // Mix bits with XOR-shift for decent spread.
    let mut s = seed;
    s ^= s >> 13;
    s ^= s << 7;
    s ^= s >> 17;
    (s % 10) as u32
}

/// Resolve a single battle. All integer arithmetic, no floats.
///
/// - attacker_score = attacker_strength + random_d10(seed) * 10
/// - defender_score = defender_strength * 110 / 100 + random_d10(seed+1) * 5
/// - if attacker_score > defender_score * 120 / 100 → AttackerAdvances
/// - if attacker_score > defender_score → Stalemate
/// - if attacker_score <= defender_score → DefenderHolds
/// - if defender loses 50%+ strength → DefenderRoutes
/// - Casualties: loser = 15%, winner = 5%
pub fn resolve_single_battle(order: &AttackOrder, tick: u64) -> BattleEvent {
    let area_seed = {
        let bytes = order.to_area.as_str().as_bytes();
        let mut h: u64 = 0;
        for &b in bytes {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        h
    };
    let seed = area_seed ^ tick;

    let att_roll = random_d10(seed) * 10;
    let def_roll = random_d10(seed.wrapping_add(1)) * 5;

    let attacker_score = order.attacker_strength + att_roll;
    let defender_score = order.defender_strength * 110 / 100 + def_roll;

    // Determine result.
    let (result, casualties_attacker, casualties_defender) =
        if attacker_score > defender_score * 120 / 100 {
            // Attacker advances — attacker wins decisively.
            let cas_att = order.attacker_strength * 5 / 100;
            let cas_def = order.defender_strength * 15 / 100;
            // Check if defender routes (lost 50%+ strength).
            if cas_def * 100 >= order.defender_strength * 50 {
                (BattleResult::DefenderRoutes, cas_att, cas_def)
            } else {
                (BattleResult::AttackerAdvances, cas_att, cas_def)
            }
        } else if attacker_score > defender_score {
            // Stalemate — slight attacker edge, both dig in.
            let cas_att = order.attacker_strength * 5 / 100;
            let cas_def = order.defender_strength * 5 / 100;
            (BattleResult::Stalemate, cas_att, cas_def)
        } else {
            // Defender holds or attacker repulsed.
            let cas_att = order.attacker_strength * 15 / 100;
            let cas_def = order.defender_strength * 5 / 100;
            (BattleResult::DefenderHolds, cas_att, cas_def)
        };

    BattleEvent {
        area: order.to_area.clone(),
        attacker: order.attacker.clone(),
        defender: order.defender.clone(),
        attacker_strength: order.attacker_strength,
        defender_strength: order.defender_strength,
        result,
        casualties_attacker,
        casualties_defender,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fra() -> PowerId {
        PowerId::from("FRA")
    }
    fn aus() -> PowerId {
        PowerId::from("AUS")
    }
    fn gbr() -> PowerId {
        PowerId::from("GBR")
    }

    fn area(name: &str) -> AreaId {
        AreaId::from(format!("AREA_{}", name))
    }

    fn corps(name: &str) -> CorpsId {
        CorpsId::from(format!("CORPS_{}", name))
    }

    fn make_order(
        from: &str,
        to: &str,
        corps_name: &str,
        attacker: &PowerId,
        defender: &PowerId,
        att_str: u32,
        def_str: u32,
    ) -> AttackOrder {
        AttackOrder {
            from_area: area(from),
            to_area: area(to),
            corps_id: corps(corps_name),
            attacker: attacker.clone(),
            defender: defender.clone(),
            attacker_strength: att_str,
            defender_strength: def_str,
        }
    }

    // ── 1. FrontLineManager creation ──

    #[test]
    fn new_manager_is_empty() {
        let mgr = FrontLineManager::new();
        assert!(mgr.get_front_lines().is_empty());
        assert!(mgr.get_attack_orders().is_empty());
        assert!(mgr.get_recent_battles().is_empty());
    }

    // ── 2. Issue attack order creates front line ──

    #[test]
    fn issue_order_creates_front_line() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        assert_eq!(mgr.get_front_lines().len(), 1);
        assert_eq!(mgr.get_attack_orders().len(), 1);
    }

    // ── 3. Contested area is tracked ──

    #[test]
    fn contested_area_tracked() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        let fl = mgr.get_front_lines().values().next().unwrap();
        assert!(fl.contested_areas.contains(&area("VIENNA")));
    }

    // ── 4. Multiple orders same front line ──

    #[test]
    fn multiple_orders_same_front_line() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        mgr.issue_attack_order(
            area("MUNICH"),
            area("PRAGUE"),
            corps("FRA_002"),
            fra(),
            aus(),
            90,
            70,
        );
        // Same attacker-defender pair → single front line.
        assert_eq!(mgr.get_front_lines().len(), 1);
        let fl = mgr.get_front_lines().values().next().unwrap();
        assert_eq!(fl.contested_areas.len(), 2);
    }

    // ── 5. Different front lines for different pairs ──

    #[test]
    fn different_pairs_different_front_lines() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        mgr.issue_attack_order(
            area("LONDON"),
            area("BREST"),
            corps("GBR_001"),
            gbr(),
            fra(),
            60,
            70,
        );
        assert_eq!(mgr.get_front_lines().len(), 2);
    }

    // ── 6. Resolve battles returns events ──

    #[test]
    fn resolve_returns_events() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        let events = mgr.resolve_battles(1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].area, area("VIENNA"));
        assert_eq!(events[0].attacker, fra());
        assert_eq!(events[0].defender, aus());
    }

    // ── 7. Resolve clears pending orders ──

    #[test]
    fn resolve_clears_orders() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        mgr.resolve_battles(1);
        assert!(mgr.get_attack_orders().is_empty());
    }

    // ── 8. Recent battles stored ──

    #[test]
    fn recent_battles_stored() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        mgr.resolve_battles(1);
        assert_eq!(mgr.get_recent_battles().len(), 1);
    }

    // ── 9. Deterministic resolution — same seed same result ──

    #[test]
    fn deterministic_same_seed() {
        let order = make_order("PARIS", "VIENNA", "FRA_001", &fra(), &aus(), 100, 80);
        let e1 = resolve_single_battle(&order, 42);
        let e2 = resolve_single_battle(&order, 42);
        assert_eq!(e1.result, e2.result);
        assert_eq!(e1.casualties_attacker, e2.casualties_attacker);
        assert_eq!(e1.casualties_defender, e2.casualties_defender);
    }

    // ── 10. Battle result is one of four variants ──

    #[test]
    fn battle_result_valid_variant() {
        let order = make_order("PARIS", "VIENNA", "FRA_001", &fra(), &aus(), 100, 80);
        for tick in 0..20 {
            let event = resolve_single_battle(&order, tick);
            match event.result {
                BattleResult::AttackerAdvances
                | BattleResult::Stalemate
                | BattleResult::DefenderHolds
                | BattleResult::DefenderRoutes => {}
            }
        }
    }

    // ── 11. Overwhelmed attacker: defender much stronger ──

    #[test]
    fn weak_attacker_usually_loses() {
        let order = make_order("PARIS", "VIENNA", "FRA_001", &fra(), &aus(), 10, 500);
        let mut defender_holds = 0;
        for tick in 0..50 {
            let event = resolve_single_battle(&order, tick);
            if event.result == BattleResult::DefenderHolds {
                defender_holds += 1;
            }
        }
        // With 10 vs 500, defender should hold most of the time.
        assert!(
            defender_holds > 30,
            "defender held {} out of 50",
            defender_holds
        );
    }

    // ── 12. Overwhelming attacker: attacker much stronger ──

    #[test]
    fn strong_attacker_usually_wins() {
        let order = make_order("PARIS", "VIENNA", "FRA_001", &fra(), &aus(), 500, 10);
        let mut attacker_wins = 0;
        for tick in 0..50 {
            let event = resolve_single_battle(&order, tick);
            if event.result == BattleResult::AttackerAdvances
                || event.result == BattleResult::DefenderRoutes
            {
                attacker_wins += 1;
            }
        }
        assert!(
            attacker_wins > 30,
            "attacker won {} out of 50",
            attacker_wins
        );
    }

    // ── 13. Casualties are non-negative ──

    #[test]
    fn casualties_non_negative() {
        let order = make_order("PARIS", "VIENNA", "FRA_001", &fra(), &aus(), 100, 80);
        for tick in 0..20 {
            let event = resolve_single_battle(&order, tick);
            // casualties are u32, so always >= 0, but verify they're reasonable.
            assert!(event.casualties_attacker <= event.attacker_strength);
            assert!(event.casualties_defender <= event.defender_strength);
        }
    }

    // ── 14. JSON serialization round-trip ──

    #[test]
    fn json_round_trip() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        mgr.resolve_battles(1);

        let fl_json = mgr.get_front_lines_json();
        let parsed: Vec<FrontLine> = serde_json::from_str(&fl_json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].attacker, fra());

        let ev_json = mgr.get_battle_events_json();
        let parsed_ev: Vec<BattleEvent> = serde_json::from_str(&ev_json).unwrap();
        assert_eq!(parsed_ev.len(), 1);
    }

    // ── 15. Pressure updates after resolve ──

    #[test]
    fn pressure_updates_on_resolve() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        let events = mgr.resolve_battles(1);
        let fl = mgr.get_front_lines().values().next().unwrap();
        let pressure = fl.pressure.get(&area("VIENNA")).copied().unwrap_or(0);
        // Pressure should have moved from initial 0.
        match events[0].result {
            BattleResult::AttackerAdvances => assert!(pressure > 0),
            BattleResult::Stalemate => assert!(pressure > 0),
            BattleResult::DefenderHolds => assert!(pressure < 0),
            BattleResult::DefenderRoutes => assert!(pressure > 0),
        }
    }

    // ── 16. Pressure clamped to [-100, 100] ──

    #[test]
    fn pressure_clamped() {
        let mut mgr = FrontLineManager::new();
        // Issue many orders to the same area to push pressure.
        for tick in 0..20u64 {
            mgr.issue_attack_order(
                area("PARIS"),
                area("VIENNA"),
                corps("FRA_001"),
                fra(),
                aus(),
                500,
                10,
            );
            mgr.resolve_battles(tick);
        }
        for fl in mgr.get_front_lines().values() {
            for &p in fl.pressure.values() {
                assert!(p >= -100 && p <= 100, "pressure out of range: {}", p);
            }
        }
    }

    // ── 17. random_d10 range check ──

    #[test]
    fn random_d10_range() {
        for seed in 0..1000u64 {
            let v = random_d10(seed);
            assert!(v < 10, "random_d10({}) = {} (expected < 10)", seed, v);
        }
    }

    // ── 18. Front line ID format ──

    #[test]
    fn front_line_id_format() {
        let mut mgr = FrontLineManager::new();
        mgr.issue_attack_order(
            area("PARIS"),
            area("VIENNA"),
            corps("FRA_001"),
            fra(),
            aus(),
            100,
            80,
        );
        let fl = mgr.get_front_lines().values().next().unwrap();
        assert!(
            fl.id.as_str().starts_with("FRONT_"),
            "ID should start with FRONT_"
        );
        assert!(fl.id.as_str().contains("FRA"));
        assert!(fl.id.as_str().contains("AUS"));
    }
}
