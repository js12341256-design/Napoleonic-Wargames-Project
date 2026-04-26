//! Production & manpower economy system.
//!
//! Standalone real-time economy that runs alongside the turn-based
//! economic phase in [`crate::economy`].  Tracks treasury, income,
//! expenditure, manpower, factories, and war exhaustion per power.
//!
//! HARD RULES (PROMPT.md §0):
//! - No floats.
//! - No HashMap — BTreeMap/BTreeSet only.
//! - No wall-clock time.

use gc1805_core_schema::ids::PowerId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ─── Core types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerEconomy {
    pub power: PowerId,
    pub treasury: i32,
    pub income_per_day: u32,
    pub expenditure_per_day: u32,
    pub manpower_pool: u32,
    pub manpower_cap: u32,
    pub manpower_recovery: u32, // per month (integer)
    pub factories: u32,
    pub war_exhaustion: u8, // 0-100
}

pub type EconomyRegistry = BTreeMap<PowerId, PowerEconomy>;

// ─── Starting economies (historical ballpark, 1805) ───────────────────

pub fn default_economies() -> EconomyRegistry {
    let entries: Vec<(&str, i32, u32, u32, u32, u32, u32, u8)> = vec![
        // (id, treasury, income/day, manpower, manpower_cap, recovery/mo, factories, war_exh)
        ("FRA", 5000, 120, 650_000, 650_000, 8000, 8, 0),
        ("GBR", 15000, 80, 150_000, 150_000, 2000, 15, 0),
        ("RUS", 2000, 60, 900_000, 900_000, 10000, 3, 0),
        ("AUS", 3000, 70, 400_000, 400_000, 5000, 5, 0),
        ("PRU", 2500, 50, 200_000, 200_000, 3000, 4, 0),
        ("OTT", 4000, 55, 300_000, 300_000, 4000, 2, 0),
        ("SPA", 6000, 65, 180_000, 180_000, 2500, 3, 0),
    ];

    let mut registry = BTreeMap::new();
    for (id, treasury, income, mp, mp_cap, recovery, factories, we) in entries {
        let power = PowerId::from(id);
        registry.insert(
            power.clone(),
            PowerEconomy {
                power,
                treasury,
                income_per_day: income,
                expenditure_per_day: default_expenditure(id),
                manpower_pool: mp,
                manpower_cap: mp_cap,
                manpower_recovery: recovery,
                factories,
                war_exhaustion: we,
            },
        );
    }
    registry
}

/// Default daily expenditure breakdown (army + navy + maintenance).
fn default_expenditure(power_id: &str) -> u32 {
    match power_id {
        "FRA" => 80,  // large army
        "GBR" => 60,  // large navy
        "RUS" => 45,
        "AUS" => 50,
        "PRU" => 35,
        "OTT" => 40,
        "SPA" => 45,
        _ => 30,
    }
}

// ─── Economy advancement ──────────────────────────────────────────────

/// Advance a single power's economy by `days` days.
/// Treasury changes by (income - expenditure) * days.
/// Manpower recovers proportionally (recovery is per 30-day month).
pub fn advance_economy(economy: &mut PowerEconomy, days: u32) {
    let net_daily = economy.income_per_day as i32 - economy.expenditure_per_day as i32;
    economy.treasury += net_daily * days as i32;

    // Manpower recovery: recovery_per_month * days / 30
    let recovered = (economy.manpower_recovery as u64 * days as u64 / 30) as u32;
    economy.manpower_pool = economy
        .manpower_pool
        .saturating_add(recovered)
        .min(economy.manpower_cap);
}

/// Advance all economies in the registry.
pub fn advance_all_economies(registry: &mut EconomyRegistry, days: u32) {
    for economy in registry.values_mut() {
        advance_economy(economy, days);
    }
}

/// Check whether a power can afford to recruit a unit.
pub fn can_recruit(economy: &PowerEconomy, unit_cost_manpower: u32, unit_cost_gold: u32) -> bool {
    economy.manpower_pool >= unit_cost_manpower && economy.treasury >= unit_cost_gold as i32
}

/// Spend manpower and gold. Returns Err if insufficient resources.
pub fn spend_resources(
    economy: &mut PowerEconomy,
    manpower: u32,
    gold: u32,
) -> Result<(), &'static str> {
    if economy.manpower_pool < manpower {
        return Err("insufficient manpower");
    }
    if economy.treasury < gold as i32 {
        return Err("insufficient treasury");
    }
    economy.manpower_pool -= manpower;
    economy.treasury -= gold as i32;
    Ok(())
}

/// Record casualties: increases war exhaustion by 1 per 10,000 casualties.
pub fn record_casualties(economy: &mut PowerEconomy, casualties: u32) {
    let we_increase = (casualties / 10_000) as u8;
    economy.war_exhaustion = economy.war_exhaustion.saturating_add(we_increase).min(100);
}

// ─── JSON serialization ───────────────────────────────────────────────

pub fn economies_to_json(registry: &EconomyRegistry) -> String {
    serde_json::to_string(registry).unwrap_or_default()
}

pub fn power_economy_to_json(registry: &EconomyRegistry, power_id: &PowerId) -> String {
    match registry.get(power_id) {
        Some(economy) => serde_json::to_string(economy).unwrap_or_default(),
        None => "null".to_string(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fra() -> PowerId {
        PowerId::from("FRA")
    }

    fn france_economy() -> PowerEconomy {
        PowerEconomy {
            power: fra(),
            treasury: 5000,
            income_per_day: 120,
            expenditure_per_day: 80,
            manpower_pool: 650_000,
            manpower_cap: 650_000,
            manpower_recovery: 8000,
            factories: 8,
            war_exhaustion: 0,
        }
    }

    #[test]
    fn default_economies_has_seven_powers() {
        let reg = default_economies();
        assert_eq!(reg.len(), 7);
        assert!(reg.contains_key(&PowerId::from("FRA")));
        assert!(reg.contains_key(&PowerId::from("GBR")));
        assert!(reg.contains_key(&PowerId::from("RUS")));
        assert!(reg.contains_key(&PowerId::from("AUS")));
        assert!(reg.contains_key(&PowerId::from("PRU")));
        assert!(reg.contains_key(&PowerId::from("OTT")));
        assert!(reg.contains_key(&PowerId::from("SPA")));
    }

    #[test]
    fn france_starting_values() {
        let reg = default_economies();
        let fra = &reg[&PowerId::from("FRA")];
        assert_eq!(fra.treasury, 5000);
        assert_eq!(fra.income_per_day, 120);
        assert_eq!(fra.manpower_pool, 650_000);
        assert_eq!(fra.factories, 8);
        assert_eq!(fra.war_exhaustion, 0);
    }

    #[test]
    fn advance_economy_treasury_increases() {
        let mut eco = france_economy();
        advance_economy(&mut eco, 10);
        // net = 120 - 80 = 40/day * 10 = +400
        assert_eq!(eco.treasury, 5400);
    }

    #[test]
    fn advance_economy_treasury_can_go_negative() {
        let mut eco = france_economy();
        eco.expenditure_per_day = 200; // net = 120 - 200 = -80/day
        eco.treasury = 100;
        advance_economy(&mut eco, 10);
        assert_eq!(eco.treasury, 100 - 800); // -700
    }

    #[test]
    fn advance_economy_manpower_recovers() {
        let mut eco = france_economy();
        eco.manpower_pool = 600_000;
        advance_economy(&mut eco, 30);
        // recovery: 8000 * 30 / 30 = 8000
        assert_eq!(eco.manpower_pool, 608_000);
    }

    #[test]
    fn advance_economy_manpower_capped() {
        let mut eco = france_economy();
        eco.manpower_pool = 649_000;
        advance_economy(&mut eco, 30);
        // would recover 8000, but capped at 650,000
        assert_eq!(eco.manpower_pool, 650_000);
    }

    #[test]
    fn can_recruit_true() {
        let eco = france_economy();
        assert!(can_recruit(&eco, 10_000, 500));
    }

    #[test]
    fn can_recruit_false_manpower() {
        let eco = france_economy();
        assert!(!can_recruit(&eco, 700_000, 100));
    }

    #[test]
    fn can_recruit_false_gold() {
        let eco = france_economy();
        assert!(!can_recruit(&eco, 1000, 10_000));
    }

    #[test]
    fn spend_resources_ok() {
        let mut eco = france_economy();
        let result = spend_resources(&mut eco, 10_000, 500);
        assert!(result.is_ok());
        assert_eq!(eco.manpower_pool, 640_000);
        assert_eq!(eco.treasury, 4500);
    }

    #[test]
    fn spend_resources_insufficient_manpower() {
        let mut eco = france_economy();
        let result = spend_resources(&mut eco, 700_000, 100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "insufficient manpower");
        // No state change on failure
        assert_eq!(eco.manpower_pool, 650_000);
    }

    #[test]
    fn spend_resources_insufficient_gold() {
        let mut eco = france_economy();
        let result = spend_resources(&mut eco, 1000, 10_000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "insufficient treasury");
    }

    #[test]
    fn record_casualties_increases_war_exhaustion() {
        let mut eco = france_economy();
        record_casualties(&mut eco, 50_000);
        assert_eq!(eco.war_exhaustion, 5);
    }

    #[test]
    fn record_casualties_capped_at_100() {
        let mut eco = france_economy();
        eco.war_exhaustion = 95;
        record_casualties(&mut eco, 200_000);
        assert_eq!(eco.war_exhaustion, 100);
    }

    #[test]
    fn advance_all_economies_updates_all() {
        let mut reg = default_economies();
        let fra_treasury_before = reg[&PowerId::from("FRA")].treasury;
        let gbr_treasury_before = reg[&PowerId::from("GBR")].treasury;
        advance_all_economies(&mut reg, 1);
        // France: net = 120 - 80 = +40
        assert_eq!(reg[&PowerId::from("FRA")].treasury, fra_treasury_before + 40);
        // Britain: net = 80 - 60 = +20
        assert_eq!(reg[&PowerId::from("GBR")].treasury, gbr_treasury_before + 20);
    }

    #[test]
    fn economies_to_json_roundtrip() {
        let reg = default_economies();
        let json = economies_to_json(&reg);
        let parsed: EconomyRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 7);
        assert_eq!(parsed[&PowerId::from("FRA")].treasury, 5000);
    }

    #[test]
    fn power_economy_to_json_known() {
        let reg = default_economies();
        let json = power_economy_to_json(&reg, &PowerId::from("FRA"));
        let parsed: PowerEconomy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.treasury, 5000);
    }

    #[test]
    fn power_economy_to_json_unknown() {
        let reg = default_economies();
        let json = power_economy_to_json(&reg, &PowerId::from("XYZ"));
        assert_eq!(json, "null");
    }

    #[test]
    fn btreemap_deterministic_order() {
        let reg = default_economies();
        let keys: Vec<_> = reg.keys().collect();
        // BTreeMap iterates in sorted order
        assert_eq!(keys[0].as_str(), "AUS");
        assert_eq!(keys[1].as_str(), "FRA");
        assert_eq!(keys[2].as_str(), "GBR");
    }
}
