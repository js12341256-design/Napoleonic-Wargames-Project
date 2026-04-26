//! Division Designer for Grand Campaign 1805.
//!
//! Templates define unit composition and battle tactics.

use std::collections::BTreeMap;

use gc1805_core_schema::ids::{DivisionTemplateId, PowerId};
use serde::{Deserialize, Serialize};

/// Battle tactic affecting combat modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleTactic {
    /// Bonus attack, penalty defense.
    Column,
    /// Balanced.
    Line,
    /// Cavalry immunity, slow movement.
    Square,
    /// Harassment, low casualties.
    SkirmishScreen,
}

/// A division template defining unit composition and doctrine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivisionTemplate {
    pub id: DivisionTemplateId,
    pub name: String,
    pub power: PowerId,
    pub battalions: u8,
    pub cavalry_squadrons: u8,
    pub artillery_batteries: u8,
    pub tactic: BattleTactic,
}

/// Computed stats for a division template (all integer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivisionStats {
    pub attack_strength: u32,
    pub defense_strength: u32,
    pub movement_speed: u32,
    pub supply_consumption: u32,
}

impl DivisionTemplate {
    /// Base combat power before tactic modifiers.
    fn base_power(&self) -> u32 {
        let inf = self.battalions as u32 * 100;
        let cav = self.cavalry_squadrons as u32 * 80;
        let art = self.artillery_batteries as u32 * 150;
        inf + cav + art
    }

    /// Compute attack strength with tactic modifier.
    pub fn attack_strength(&self) -> u32 {
        let base = self.base_power();
        match self.tactic {
            // +20% attack
            BattleTactic::Column => base + base / 5,
            BattleTactic::Line => base,
            // -30% attack
            BattleTactic::Square => base - base * 3 / 10,
            // -20% attack
            BattleTactic::SkirmishScreen => base - base / 5,
        }
    }

    /// Compute defense strength with tactic modifier.
    pub fn defense_strength(&self) -> u32 {
        let base = self.base_power();
        match self.tactic {
            // -10% defense
            BattleTactic::Column => base - base / 10,
            BattleTactic::Line => base,
            // +30% defense
            BattleTactic::Square => base + base * 3 / 10,
            // +10% defense
            BattleTactic::SkirmishScreen => base + base / 10,
        }
    }

    /// Movement speed: base 3, -1 per 3 artillery, +1 if cavalry > infantry.
    pub fn movement_speed(&self) -> u32 {
        let mut speed: u32 = 3;
        let art_penalty = self.artillery_batteries as u32 / 3;
        speed = speed.saturating_sub(art_penalty);
        if self.cavalry_squadrons > self.battalions {
            speed += 1;
        }
        // Square tactic is slow: -1
        if self.tactic == BattleTactic::Square {
            speed = speed.saturating_sub(1);
        }
        speed
    }

    /// Supply consumption based on unit counts.
    pub fn supply_consumption(&self) -> u32 {
        let personnel = (self.battalions as u32 + self.cavalry_squadrons as u32) * 10;
        let artillery = self.artillery_batteries as u32 * 25;
        personnel + artillery
    }

    /// Compute all stats at once.
    pub fn stats(&self) -> DivisionStats {
        DivisionStats {
            attack_strength: self.attack_strength(),
            defense_strength: self.defense_strength(),
            movement_speed: self.movement_speed(),
            supply_consumption: self.supply_consumption(),
        }
    }
}

/// Registry of division templates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DivisionRegistry {
    pub templates: BTreeMap<DivisionTemplateId, DivisionTemplate>,
    next_id: u32,
}

impl DivisionRegistry {
    /// Create a registry with default templates for all major powers.
    pub fn with_defaults() -> Self {
        let mut reg = Self {
            templates: BTreeMap::new(),
            next_id: 100,
        };

        let defaults: &[(&str, &str, &str, u8, u8, u8, BattleTactic)] = &[
            ("DIVTPL_FRA_DEFAULT", "Grande Armée Corps", "FRA", 8, 3, 2, BattleTactic::Column),
            ("DIVTPL_GBR_DEFAULT", "British Division", "GBR", 6, 1, 2, BattleTactic::Line),
            ("DIVTPL_RUS_DEFAULT", "Russian Corps", "RUS", 10, 2, 1, BattleTactic::Column),
            ("DIVTPL_AUS_DEFAULT", "Austrian Corps", "AUS", 8, 2, 2, BattleTactic::Line),
            ("DIVTPL_PRU_DEFAULT", "Prussian Corps", "PRU", 7, 3, 2, BattleTactic::Column),
        ];

        for &(id, name, power, inf, cav, art, tactic) in defaults {
            let tpl_id = DivisionTemplateId::from(id);
            reg.templates.insert(
                tpl_id.clone(),
                DivisionTemplate {
                    id: tpl_id,
                    name: name.to_owned(),
                    power: PowerId::from(power),
                    battalions: inf,
                    cavalry_squadrons: cav,
                    artillery_batteries: art,
                    tactic,
                },
            );
        }

        reg
    }

    /// Create a new division template from JSON. Returns the new template as JSON.
    pub fn create_from_json(&mut self, json: &str) -> Result<String, String> {
        #[derive(Deserialize)]
        struct Input {
            name: String,
            power: String,
            battalions: u8,
            cavalry_squadrons: u8,
            artillery_batteries: u8,
            tactic: BattleTactic,
        }

        let input: Input = serde_json::from_str(json).map_err(|e| e.to_string())?;

        let id = DivisionTemplateId::from(format!("DIVTPL_CUSTOM_{}", self.next_id));
        self.next_id += 1;

        let tpl = DivisionTemplate {
            id: id.clone(),
            name: input.name,
            power: PowerId::from(input.power),
            battalions: input.battalions,
            cavalry_squadrons: input.cavalry_squadrons,
            artillery_batteries: input.artillery_batteries,
            tactic: input.tactic,
        };

        let result = serde_json::to_string(&tpl).map_err(|e| e.to_string())?;
        self.templates.insert(id, tpl);
        Ok(result)
    }

    /// Serialize all templates to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.templates).unwrap_or_default()
    }

    /// Get stats for a template by ID, returned as JSON.
    pub fn stats_json(&self, template_id: &DivisionTemplateId) -> Result<String, String> {
        let tpl = self
            .templates
            .get(template_id)
            .ok_or_else(|| format!("Template {} not found", template_id))?;
        serde_json::to_string(&tpl.stats()).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_five_templates() {
        let reg = DivisionRegistry::with_defaults();
        assert_eq!(reg.templates.len(), 5);
    }

    #[test]
    fn french_corps_stats() {
        let reg = DivisionRegistry::with_defaults();
        let fra = reg
            .templates
            .get(&DivisionTemplateId::from("DIVTPL_FRA_DEFAULT"))
            .unwrap();
        // Base: 8*100 + 3*80 + 2*150 = 800+240+300 = 1340
        // Column: +20% attack = 1340 + 268 = 1608
        assert_eq!(fra.attack_strength(), 1608);
        // Column: -10% defense = 1340 - 134 = 1206
        assert_eq!(fra.defense_strength(), 1206);
        // Movement: 3, art=2 => 2/3=0 penalty, cav(3) < inf(8) => no bonus => 3
        assert_eq!(fra.movement_speed(), 3);
        // Supply: (8+3)*10 + 2*25 = 110+50 = 160
        assert_eq!(fra.supply_consumption(), 160);
    }

    #[test]
    fn british_division_line_tactic() {
        let reg = DivisionRegistry::with_defaults();
        let gbr = reg
            .templates
            .get(&DivisionTemplateId::from("DIVTPL_GBR_DEFAULT"))
            .unwrap();
        // Base: 6*100 + 1*80 + 2*150 = 600+80+300 = 980
        // Line: no modifier
        assert_eq!(gbr.attack_strength(), 980);
        assert_eq!(gbr.defense_strength(), 980);
    }

    #[test]
    fn square_tactic_modifiers() {
        let tpl = DivisionTemplate {
            id: DivisionTemplateId::from("DIVTPL_TEST"),
            name: "Test".to_owned(),
            power: PowerId::from("FRA"),
            battalions: 10,
            cavalry_squadrons: 0,
            artillery_batteries: 0,
            tactic: BattleTactic::Square,
        };
        // Base: 1000
        // Square: -30% attack = 700, +30% defense = 1300
        assert_eq!(tpl.attack_strength(), 700);
        assert_eq!(tpl.defense_strength(), 1300);
        // Square: movement -1 => base 3 - 1 = 2
        assert_eq!(tpl.movement_speed(), 2);
    }

    #[test]
    fn skirmish_screen_modifiers() {
        let tpl = DivisionTemplate {
            id: DivisionTemplateId::from("DIVTPL_TEST"),
            name: "Test".to_owned(),
            power: PowerId::from("FRA"),
            battalions: 10,
            cavalry_squadrons: 0,
            artillery_batteries: 0,
            tactic: BattleTactic::SkirmishScreen,
        };
        // Base: 1000
        // SkirmishScreen: -20% attack = 800, +10% defense = 1100
        assert_eq!(tpl.attack_strength(), 800);
        assert_eq!(tpl.defense_strength(), 1100);
    }

    #[test]
    fn create_custom_template() {
        let mut reg = DivisionRegistry::with_defaults();
        let json = r#"{"name":"Custom","power":"FRA","battalions":5,"cavalry_squadrons":5,"artillery_batteries":1,"tactic":"Column"}"#;
        let result = reg.create_from_json(json);
        assert!(result.is_ok());
        assert_eq!(reg.templates.len(), 6);
    }

    #[test]
    fn stats_json_for_known_template() {
        let reg = DivisionRegistry::with_defaults();
        let result = reg.stats_json(&DivisionTemplateId::from("DIVTPL_FRA_DEFAULT"));
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("attack_strength"));
    }

    #[test]
    fn stats_json_unknown_template_fails() {
        let reg = DivisionRegistry::with_defaults();
        let result = reg.stats_json(&DivisionTemplateId::from("DIVTPL_NOPE"));
        assert!(result.is_err());
    }

    #[test]
    fn cavalry_speed_bonus() {
        let tpl = DivisionTemplate {
            id: DivisionTemplateId::from("DIVTPL_TEST"),
            name: "Cavalry".to_owned(),
            power: PowerId::from("FRA"),
            battalions: 1,
            cavalry_squadrons: 5,
            artillery_batteries: 0,
            tactic: BattleTactic::Line,
        };
        // cav(5) > inf(1) => +1 => speed = 4
        assert_eq!(tpl.movement_speed(), 4);
    }

    #[test]
    fn artillery_slows_movement() {
        let tpl = DivisionTemplate {
            id: DivisionTemplateId::from("DIVTPL_TEST"),
            name: "Heavy Art".to_owned(),
            power: PowerId::from("FRA"),
            battalions: 5,
            cavalry_squadrons: 0,
            artillery_batteries: 9,
            tactic: BattleTactic::Line,
        };
        // 9/3 = 3 penalty, base 3 - 3 = 0
        assert_eq!(tpl.movement_speed(), 0);
    }

    #[test]
    fn to_json_round_trip() {
        let reg = DivisionRegistry::with_defaults();
        let json = reg.to_json();
        let parsed: BTreeMap<DivisionTemplateId, DivisionTemplate> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 5);
    }
}
