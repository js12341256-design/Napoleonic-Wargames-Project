//! Politics & Legitimacy system for Grand Campaign 1805.
//!
//! Each major power has legitimacy (0-100), stability (-3 to +3),
//! government type, ruling faction, and faction support values.
//! Stability affects income, manpower recovery, and revolt risk.

use std::collections::{BTreeMap, BTreeSet};

use gc1805_core_schema::ids::PowerId;
use serde::{Deserialize, Serialize};

/// Government type of a major power.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Government {
    Empire,
    AbsoluteMonarchy,
    ConstitutionalMonarchy,
    Republic,
}

/// Political factions vying for influence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Faction {
    Military,
    Nobility,
    Clergy,
    Merchants,
    Peasantry,
    Revolutionaries,
}

/// Modifiers derived from current stability level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StabilityEffects {
    /// Income modifier in percentage points (e.g. +15 means +15%).
    pub income_modifier: i8,
    /// Manpower recovery modifier in percentage points.
    pub manpower_modifier: i8,
    /// Monthly revolt chance in percentage points (0 = no revolt risk from stability).
    pub revolt_chance: u8,
    /// Whether civil war is possible at this stability level.
    pub civil_war_risk: bool,
}

/// The politics state of a single major power.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPolitics {
    pub power: PowerId,
    /// 0-100. Legitimacy of the current government.
    pub legitimacy: u8,
    /// -3 to +3, HoI4-style stability.
    pub stability: i8,
    pub government: Government,
    pub ruling_faction: Faction,
    /// Faction -> popularity 0-100.
    pub faction_support: BTreeMap<Faction, u8>,
    /// Powers that are puppets of this power.
    pub puppets: BTreeSet<PowerId>,
    /// If this power is a puppet, its overlord.
    pub overlord: Option<PowerId>,
}

impl PowerPolitics {
    /// Compute daily legitimacy change.
    ///
    /// - Stability +3: +1/month, +2: ~+0.5/month, +1: ~+0.3/month
    /// - Stability  0: 0
    /// - Stability -1: -0.5/month, -2: -1/month, -3: -2/month
    ///
    /// `wars_ongoing` is the number of active wars, `winning_wars` whether
    /// at least one war is being won.
    ///
    /// Returns monthly legitimacy change (apply by dividing by 30 for daily).
    pub fn monthly_legitimacy_change(&self, wars_ongoing: u8, winning_wars: bool) -> i8 {
        let base: i8 = match self.stability {
            3 => 1,
            2 => 1,
            1 => 0,
            0 => 0,
            -1 => -1,
            -2 => -1,
            _ if self.stability < -2 => -2,
            _ => 0,
        };
        let war_penalty = -(wars_ongoing as i8);
        let win_bonus: i8 = if winning_wars { 2 } else { 0 };
        base.saturating_add(war_penalty).saturating_add(win_bonus)
    }

    /// Compute the stability effects (income/manpower modifiers, revolt risk).
    pub fn stability_effects(&self) -> StabilityEffects {
        match self.stability {
            3 => StabilityEffects { income_modifier: 15, manpower_modifier: 10, revolt_chance: 0, civil_war_risk: false },
            2 => StabilityEffects { income_modifier: 10, manpower_modifier: 5, revolt_chance: 0, civil_war_risk: false },
            1 => StabilityEffects { income_modifier: 5, manpower_modifier: 0, revolt_chance: 0, civil_war_risk: false },
            0 => StabilityEffects { income_modifier: 0, manpower_modifier: 0, revolt_chance: 0, civil_war_risk: false },
            -1 => StabilityEffects { income_modifier: -10, manpower_modifier: 0, revolt_chance: 5, civil_war_risk: false },
            -2 => StabilityEffects { income_modifier: -20, manpower_modifier: 0, revolt_chance: 15, civil_war_risk: false },
            _ if self.stability <= -3 => StabilityEffects { income_modifier: -30, manpower_modifier: 0, revolt_chance: 30, civil_war_risk: true },
            _ => StabilityEffects { income_modifier: 0, manpower_modifier: 0, revolt_chance: 0, civil_war_risk: false },
        }
    }

    /// Clamp stability to valid range [-3, +3].
    pub fn clamp_stability(&mut self) {
        self.stability = self.stability.clamp(-3, 3);
    }

    /// Clamp legitimacy to valid range [0, 100].
    pub fn clamp_legitimacy(&mut self) {
        self.legitimacy = self.legitimacy.min(100);
    }
}

/// Registry of all powers' political states.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoliticsRegistry {
    pub powers: BTreeMap<PowerId, PowerPolitics>,
}

impl PoliticsRegistry {
    /// Create a registry with all 1805 starting values.
    pub fn with_historical() -> Self {
        let mut powers = BTreeMap::new();

        let france = PowerPolitics {
            power: PowerId::from("FRA"),
            legitimacy: 85,
            stability: 2,
            government: Government::Empire,
            ruling_faction: Faction::Military,
            faction_support: BTreeMap::from([
                (Faction::Military, 60),
                (Faction::Merchants, 40),
                (Faction::Clergy, 30),
            ]),
            puppets: BTreeSet::new(),
            overlord: None,
        };
        powers.insert(PowerId::from("FRA"), france);

        let britain = PowerPolitics {
            power: PowerId::from("GBR"),
            legitimacy: 90,
            stability: 3,
            government: Government::ConstitutionalMonarchy,
            ruling_faction: Faction::Merchants,
            faction_support: BTreeMap::from([
                (Faction::Nobility, 70),
                (Faction::Merchants, 80),
            ]),
            puppets: BTreeSet::new(),
            overlord: None,
        };
        powers.insert(PowerId::from("GBR"), britain);

        let russia = PowerPolitics {
            power: PowerId::from("RUS"),
            legitimacy: 75,
            stability: 1,
            government: Government::AbsoluteMonarchy,
            ruling_faction: Faction::Nobility,
            faction_support: BTreeMap::from([
                (Faction::Nobility, 80),
                (Faction::Military, 60),
            ]),
            puppets: BTreeSet::new(),
            overlord: None,
        };
        powers.insert(PowerId::from("RUS"), russia);

        let austria = PowerPolitics {
            power: PowerId::from("AUS"),
            legitimacy: 80,
            stability: 1,
            government: Government::AbsoluteMonarchy,
            ruling_faction: Faction::Nobility,
            faction_support: BTreeMap::from([
                (Faction::Nobility, 75),
                (Faction::Clergy, 65),
            ]),
            puppets: BTreeSet::new(),
            overlord: None,
        };
        powers.insert(PowerId::from("AUS"), austria);

        let prussia = PowerPolitics {
            power: PowerId::from("PRU"),
            legitimacy: 70,
            stability: 0,
            government: Government::AbsoluteMonarchy,
            ruling_faction: Faction::Military,
            faction_support: BTreeMap::from([
                (Faction::Military, 80),
                (Faction::Nobility, 60),
            ]),
            puppets: BTreeSet::new(),
            overlord: None,
        };
        powers.insert(PowerId::from("PRU"), prussia);

        let ottoman = PowerPolitics {
            power: PowerId::from("OTT"),
            legitimacy: 60,
            stability: -1,
            government: Government::AbsoluteMonarchy,
            ruling_faction: Faction::Clergy,
            faction_support: BTreeMap::from([
                (Faction::Military, 50),
                (Faction::Clergy, 70),
            ]),
            puppets: BTreeSet::new(),
            overlord: None,
        };
        powers.insert(PowerId::from("OTT"), ottoman);

        let spain = PowerPolitics {
            power: PowerId::from("SPA"),
            legitimacy: 55,
            stability: -2,
            government: Government::AbsoluteMonarchy,
            ruling_faction: Faction::Clergy,
            faction_support: BTreeMap::from([
                (Faction::Clergy, 70),
                (Faction::Nobility, 50),
            ]),
            puppets: BTreeSet::new(),
            overlord: None,
        };
        powers.insert(PowerId::from("SPA"), spain);

        PoliticsRegistry { powers }
    }

    /// Get politics for a single power as JSON.
    pub fn power_politics_json(&self, power: &PowerId) -> String {
        self.powers
            .get(power)
            .map(|p| serde_json::to_string(p).unwrap_or_default())
            .unwrap_or_else(|| "null".to_string())
    }

    /// Get all politics as JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.powers).unwrap_or_default()
    }

    /// Change the ruling faction for a power.
    pub fn change_ruling_faction(&mut self, power: &PowerId, faction: Faction) -> Result<(), String> {
        let politics = self.powers.get_mut(power).ok_or_else(|| format!("Unknown power: {}", power.as_str()))?;
        politics.ruling_faction = faction;
        Ok(())
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fra() -> PowerId { PowerId::from("FRA") }
    fn gbr() -> PowerId { PowerId::from("GBR") }

    #[test]
    fn historical_registry_has_seven_powers() {
        let reg = PoliticsRegistry::with_historical();
        assert_eq!(reg.powers.len(), 7);
    }

    #[test]
    fn france_starting_values() {
        let reg = PoliticsRegistry::with_historical();
        let fra = &reg.powers[&fra()];
        assert_eq!(fra.legitimacy, 85);
        assert_eq!(fra.stability, 2);
        assert_eq!(fra.government, Government::Empire);
        assert_eq!(fra.ruling_faction, Faction::Military);
        assert_eq!(fra.faction_support[&Faction::Military], 60);
    }

    #[test]
    fn stability_effects_positive() {
        let reg = PoliticsRegistry::with_historical();
        let gbr = &reg.powers[&gbr()];
        let effects = gbr.stability_effects();
        assert_eq!(effects.income_modifier, 15);
        assert_eq!(effects.manpower_modifier, 10);
        assert_eq!(effects.revolt_chance, 0);
        assert!(!effects.civil_war_risk);
    }

    #[test]
    fn stability_effects_negative() {
        let reg = PoliticsRegistry::with_historical();
        let spa = &reg.powers[&PowerId::from("SPA")];
        let effects = spa.stability_effects();
        assert_eq!(effects.income_modifier, -20);
        assert_eq!(effects.revolt_chance, 15);
        assert!(!effects.civil_war_risk);
    }

    #[test]
    fn stability_effects_civil_war() {
        let mut politics = PoliticsRegistry::with_historical().powers.remove(&fra()).unwrap();
        politics.stability = -3;
        let effects = politics.stability_effects();
        assert_eq!(effects.income_modifier, -30);
        assert_eq!(effects.revolt_chance, 30);
        assert!(effects.civil_war_risk);
    }

    #[test]
    fn monthly_legitimacy_no_wars() {
        let reg = PoliticsRegistry::with_historical();
        let fra = &reg.powers[&fra()];
        // stability 2 => base +1, no wars, not winning => +1
        assert_eq!(fra.monthly_legitimacy_change(0, false), 1);
    }

    #[test]
    fn monthly_legitimacy_with_wars() {
        let reg = PoliticsRegistry::with_historical();
        let fra = &reg.powers[&fra()];
        // stability 2 => base +1, 2 wars => -2, not winning => 1-2 = -1
        assert_eq!(fra.monthly_legitimacy_change(2, false), -1);
    }

    #[test]
    fn monthly_legitimacy_winning_wars() {
        let reg = PoliticsRegistry::with_historical();
        let fra = &reg.powers[&fra()];
        // stability 2 => base +1, 1 war => -1, winning => +2: total 2
        assert_eq!(fra.monthly_legitimacy_change(1, true), 2);
    }

    #[test]
    fn change_faction_succeeds() {
        let mut reg = PoliticsRegistry::with_historical();
        assert!(reg.change_ruling_faction(&fra(), Faction::Merchants).is_ok());
        assert_eq!(reg.powers[&fra()].ruling_faction, Faction::Merchants);
    }

    #[test]
    fn change_faction_unknown_power_errors() {
        let mut reg = PoliticsRegistry::with_historical();
        let result = reg.change_ruling_faction(&PowerId::from("ZZZ"), Faction::Military);
        assert!(result.is_err());
    }

    #[test]
    fn politics_json_round_trip() {
        let reg = PoliticsRegistry::with_historical();
        let json = reg.power_politics_json(&fra());
        let parsed: PowerPolitics = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.legitimacy, 85);
        assert_eq!(parsed.stability, 2);
    }
}
