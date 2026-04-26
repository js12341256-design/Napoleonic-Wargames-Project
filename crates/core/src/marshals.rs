//! Marshals / Commanders system for Grand Campaign 1805.
//!
//! Historical marshals with traits that affect combat and logistics.

use std::collections::BTreeMap;

use gc1805_core_schema::ids::{CorpsId, MarshalId, PowerId};
use serde::{Deserialize, Serialize};

/// Combat and strategic traits for marshals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarshalTrait {
    /// +20% defense
    DefensiveGenius,
    /// +15% attack speed
    Aggressive,
    /// Doubles cavalry effectiveness
    CavalryMaster,
    /// +10% all combat
    Tactician,
    /// +25% supply range
    Logistics,
    /// +50% fort attack
    Siege,
    /// +20% naval combat
    NavalCommander,
    /// +10% morale
    InspirationalLeader,
}

/// A named historical commander with skills and traits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marshal {
    pub id: MarshalId,
    pub name: String,
    pub power: PowerId,
    pub traits: Vec<MarshalTrait>,
    pub assigned_corps: Option<CorpsId>,
    pub skill: u8,
    pub portrait_key: String,
}

/// Registry of all marshals in the game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarshalRegistry {
    pub marshals: BTreeMap<MarshalId, Marshal>,
}

impl MarshalRegistry {
    /// Create a registry populated with all historical marshals.
    pub fn with_historical() -> Self {
        let mut reg = Self::default();

        let entries: &[(&str, &str, &str, u8, &[MarshalTrait])] = &[
            // France
            ("MARSHAL_NAPOLEON", "Napoleon Bonaparte", "FRA", 10,
                &[MarshalTrait::Tactician, MarshalTrait::Aggressive, MarshalTrait::InspirationalLeader]),
            ("MARSHAL_DAVOUT", "Louis-Nicolas Davout", "FRA", 9,
                &[MarshalTrait::DefensiveGenius, MarshalTrait::Tactician]),
            ("MARSHAL_NEY", "Michel Ney", "FRA", 8,
                &[MarshalTrait::Aggressive, MarshalTrait::InspirationalLeader]),
            ("MARSHAL_MURAT", "Joachim Murat", "FRA", 8,
                &[MarshalTrait::CavalryMaster, MarshalTrait::Aggressive]),
            ("MARSHAL_SOULT", "Nicolas Jean-de-Dieu Soult", "FRA", 8,
                &[MarshalTrait::Tactician, MarshalTrait::Logistics]),
            ("MARSHAL_MASSENA", "André Masséna", "FRA", 8,
                &[MarshalTrait::DefensiveGenius]),
            ("MARSHAL_LANNES", "Jean Lannes", "FRA", 8,
                &[MarshalTrait::Aggressive]),
            ("MARSHAL_BERTHIER", "Louis-Alexandre Berthier", "FRA", 7,
                &[MarshalTrait::Logistics]),
            // Britain
            ("MARSHAL_WELLINGTON", "Arthur Wellesley, Duke of Wellington", "GBR", 9,
                &[MarshalTrait::DefensiveGenius, MarshalTrait::Tactician]),
            ("MARSHAL_NELSON", "Horatio Nelson", "GBR", 9,
                &[MarshalTrait::NavalCommander, MarshalTrait::InspirationalLeader]),
            ("MARSHAL_MOORE", "Sir John Moore", "GBR", 7,
                &[MarshalTrait::Tactician]),
            // Russia
            ("MARSHAL_KUTUZOV", "Mikhail Kutuzov", "RUS", 9,
                &[MarshalTrait::DefensiveGenius, MarshalTrait::Logistics]),
            ("MARSHAL_BAGRATION", "Pyotr Bagration", "RUS", 8,
                &[MarshalTrait::Aggressive]),
            ("MARSHAL_BARCLAY", "Barclay de Tolly", "RUS", 7,
                &[MarshalTrait::Logistics, MarshalTrait::DefensiveGenius]),
            // Austria
            ("MARSHAL_CHARLES", "Archduke Charles", "AUS", 8,
                &[MarshalTrait::DefensiveGenius, MarshalTrait::Tactician]),
            ("MARSHAL_SCHWARZENBERG", "Karl Philipp, Prince of Schwarzenberg", "AUS", 7,
                &[MarshalTrait::Logistics]),
            // Prussia
            ("MARSHAL_BLUCHER", "Gebhard Leberecht von Blücher", "PRU", 8,
                &[MarshalTrait::Aggressive, MarshalTrait::InspirationalLeader]),
            ("MARSHAL_SCHARNHORST", "Gerhard von Scharnhorst", "PRU", 7,
                &[MarshalTrait::Tactician]),
            // Ottoman
            ("MARSHAL_MEHMED", "Mehmed Ali Pasha", "OTT", 6,
                &[MarshalTrait::Siege]),
            // Spain
            ("MARSHAL_CASTANOS", "Francisco Javier Castaños", "SPA", 7,
                &[MarshalTrait::DefensiveGenius]),
        ];

        for &(id, name, power, skill, traits) in entries {
            let marshal_id = MarshalId::from(id);
            let portrait = id.to_ascii_lowercase();
            reg.marshals.insert(
                marshal_id.clone(),
                Marshal {
                    id: marshal_id,
                    name: name.to_owned(),
                    power: PowerId::from(power),
                    traits: traits.to_vec(),
                    assigned_corps: None,
                    skill,
                    portrait_key: portrait,
                },
            );
        }

        reg
    }

    /// Assign a marshal to a corps. Returns an error string if marshal not found.
    pub fn assign_marshal(
        &mut self,
        marshal_id: &MarshalId,
        corps_id: &CorpsId,
    ) -> Result<(), String> {
        let marshal = self
            .marshals
            .get_mut(marshal_id)
            .ok_or_else(|| format!("Marshal {} not found", marshal_id))?;
        marshal.assigned_corps = Some(corps_id.clone());
        Ok(())
    }

    /// Get all marshals belonging to a given power.
    pub fn get_power_marshals(&self, power_id: &PowerId) -> Vec<&Marshal> {
        self.marshals
            .values()
            .filter(|m| m.power == *power_id)
            .collect()
    }

    /// Serialize all marshals to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.marshals).unwrap_or_default()
    }

    /// Serialize marshals of a power to JSON.
    pub fn power_marshals_json(&self, power_id: &PowerId) -> String {
        let marshals: Vec<&Marshal> = self.get_power_marshals(power_id);
        serde_json::to_string(&marshals).unwrap_or_default()
    }
}

/// Compute the defense bonus percentage from a marshal's traits.
pub fn defense_bonus(marshal: &Marshal) -> u32 {
    let mut bonus: u32 = 0;
    for t in &marshal.traits {
        match t {
            MarshalTrait::DefensiveGenius => bonus += 20,
            MarshalTrait::Tactician => bonus += 10,
            _ => {}
        }
    }
    bonus
}

/// Compute the attack bonus percentage from a marshal's traits.
pub fn attack_bonus(marshal: &Marshal) -> u32 {
    let mut bonus: u32 = 0;
    for t in &marshal.traits {
        match t {
            MarshalTrait::Aggressive => bonus += 15,
            MarshalTrait::Tactician => bonus += 10,
            _ => {}
        }
    }
    bonus
}

/// Compute the morale bonus percentage from a marshal's traits.
pub fn morale_bonus(marshal: &Marshal) -> u32 {
    let mut bonus: u32 = 0;
    for t in &marshal.traits {
        if *t == MarshalTrait::InspirationalLeader {
            bonus += 10;
        }
    }
    bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_registry_has_all_marshals() {
        let reg = MarshalRegistry::with_historical();
        assert_eq!(reg.marshals.len(), 20);
    }

    #[test]
    fn napoleon_has_correct_stats() {
        let reg = MarshalRegistry::with_historical();
        let nap = reg.marshals.get(&MarshalId::from("MARSHAL_NAPOLEON")).unwrap();
        assert_eq!(nap.skill, 10);
        assert_eq!(nap.power, PowerId::from("FRA"));
        assert!(nap.traits.contains(&MarshalTrait::Tactician));
        assert!(nap.traits.contains(&MarshalTrait::Aggressive));
        assert!(nap.traits.contains(&MarshalTrait::InspirationalLeader));
    }

    #[test]
    fn french_marshals_count() {
        let reg = MarshalRegistry::with_historical();
        let french = reg.get_power_marshals(&PowerId::from("FRA"));
        assert_eq!(french.len(), 8);
    }

    #[test]
    fn assign_marshal_to_corps() {
        let mut reg = MarshalRegistry::with_historical();
        let mid = MarshalId::from("MARSHAL_DAVOUT");
        let cid = CorpsId::from("CORPS_FRA_001");
        assert!(reg.assign_marshal(&mid, &cid).is_ok());
        let davout = reg.marshals.get(&mid).unwrap();
        assert_eq!(davout.assigned_corps, Some(cid));
    }

    #[test]
    fn assign_unknown_marshal_fails() {
        let mut reg = MarshalRegistry::with_historical();
        let mid = MarshalId::from("MARSHAL_NOBODY");
        let cid = CorpsId::from("CORPS_FRA_001");
        assert!(reg.assign_marshal(&mid, &cid).is_err());
    }

    #[test]
    fn defense_bonus_for_davout() {
        let reg = MarshalRegistry::with_historical();
        let davout = reg.marshals.get(&MarshalId::from("MARSHAL_DAVOUT")).unwrap();
        // DefensiveGenius (20) + Tactician (10) = 30
        assert_eq!(defense_bonus(davout), 30);
    }

    #[test]
    fn attack_bonus_for_napoleon() {
        let reg = MarshalRegistry::with_historical();
        let nap = reg.marshals.get(&MarshalId::from("MARSHAL_NAPOLEON")).unwrap();
        // Aggressive (15) + Tactician (10) = 25
        assert_eq!(attack_bonus(nap), 25);
    }

    #[test]
    fn morale_bonus_for_ney() {
        let reg = MarshalRegistry::with_historical();
        let ney = reg.marshals.get(&MarshalId::from("MARSHAL_NEY")).unwrap();
        assert_eq!(morale_bonus(ney), 10);
    }

    #[test]
    fn to_json_round_trip() {
        let reg = MarshalRegistry::with_historical();
        let json = reg.to_json();
        let parsed: BTreeMap<MarshalId, Marshal> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 20);
    }

    #[test]
    fn power_marshals_json_is_valid() {
        let reg = MarshalRegistry::with_historical();
        let json = reg.power_marshals_json(&PowerId::from("GBR"));
        let parsed: Vec<Marshal> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 3);
    }
}
