//! National Focus Tree system.
//!
//! Each major power has a focus tree — a directed acyclic graph of
//! national focuses that grant bonuses when completed.  Only one
//! focus may be in progress at a time per power.

#![allow(clippy::float_arithmetic)] // no floats used — integer days only

use std::collections::{BTreeMap, BTreeSet};

use gc1805_core_schema::ids::PowerId;
use serde::{Deserialize, Serialize};

// ── ID type ──

/// Lightweight focus identifier (integer, not string-based).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct FocusId(pub u32);

// ── Effect enum ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FocusEffect {
    ManpowerBonus(u32),
    AttackBonus(u32),
    DefenseBonus(u32),
    SupplyRangeBonus(u32),
    DiplomaticInfluence(String, i32),
    UnlockUnit(String),
    TreasuryBonus(u32),
    NavalBonus(u32),
    ResearchBonus(u32),
}

// ── Focus node ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Focus {
    pub id: FocusId,
    pub name: String,
    pub description: String,
    pub power: PowerId,
    pub cost_days: u32,
    pub prerequisites: Vec<FocusId>,
    pub effects: Vec<FocusEffect>,
    pub x: i32,
    pub y: i32,
    pub icon: String,
    pub category: String,
}

// ── Focus Tree (per power) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTree {
    pub power: PowerId,
    pub focuses: BTreeMap<FocusId, Focus>,
    pub completed: BTreeSet<FocusId>,
    pub in_progress: Option<(FocusId, u32)>,
}

impl FocusTree {
    pub fn new(power: PowerId) -> Self {
        Self {
            power,
            focuses: BTreeMap::new(),
            completed: BTreeSet::new(),
            in_progress: None,
        }
    }

    /// Insert a focus into the tree.
    pub fn add_focus(&mut self, focus: Focus) {
        self.focuses.insert(focus.id, focus);
    }

    /// Check if all prerequisites for a focus are completed.
    pub fn prerequisites_met(&self, focus_id: FocusId) -> bool {
        match self.focuses.get(&focus_id) {
            Some(focus) => focus
                .prerequisites
                .iter()
                .all(|pre| self.completed.contains(pre)),
            None => false,
        }
    }

    /// Check if a focus is available (exists, not completed, not in-progress,
    /// and all prereqs met).
    pub fn is_available(&self, focus_id: FocusId) -> bool {
        if self.completed.contains(&focus_id) {
            return false;
        }
        if let Some((current, _)) = &self.in_progress {
            if *current == focus_id {
                return false;
            }
        }
        self.prerequisites_met(focus_id)
    }

    /// Start working on a focus.  Returns `false` if focus is not available
    /// or another focus is already in progress.
    pub fn start_focus(&mut self, focus_id: FocusId) -> bool {
        if self.in_progress.is_some() {
            return false;
        }
        if !self.is_available(focus_id) {
            return false;
        }
        let cost = self.focuses.get(&focus_id).unwrap().cost_days;
        self.in_progress = Some((focus_id, cost));
        true
    }

    /// Advance the in-progress focus by `days`.  When completed the focus
    /// is moved to `completed` and `in_progress` is cleared.
    /// Returns the list of effects if a focus just completed.
    pub fn advance(&mut self, days: u32) -> Option<Vec<FocusEffect>> {
        if let Some((fid, remaining)) = &mut self.in_progress {
            if days >= *remaining {
                let fid_done = *fid;
                self.completed.insert(fid_done);
                let effects = self
                    .focuses
                    .get(&fid_done)
                    .map(|f| f.effects.clone())
                    .unwrap_or_default();
                self.in_progress = None;
                Some(effects)
            } else {
                *remaining -= days;
                None
            }
        } else {
            None
        }
    }

    /// Serialize the tree to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ── Focus Tree Registry (all powers) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusRegistry {
    pub trees: BTreeMap<String, FocusTree>,
}

impl FocusRegistry {
    /// Create a registry with the default France and Britain focus trees.
    pub fn with_defaults() -> Self {
        let mut reg = Self {
            trees: BTreeMap::new(),
        };
        reg.trees
            .insert("FRA".to_owned(), build_france_tree());
        reg.trees
            .insert("GBR".to_owned(), build_britain_tree());
        reg
    }

    pub fn get_tree(&self, power_id: &str) -> Option<&FocusTree> {
        self.trees.get(power_id)
    }

    pub fn get_tree_mut(&mut self, power_id: &str) -> Option<&mut FocusTree> {
        self.trees.get_mut(power_id)
    }

    pub fn get_tree_json(&self, power_id: &str) -> String {
        match self.trees.get(power_id) {
            Some(tree) => tree.to_json(),
            None => "null".to_owned(),
        }
    }

    pub fn get_completed_json(&self, power_id: &str) -> String {
        match self.trees.get(power_id) {
            Some(tree) => serde_json::to_string(&tree.completed).unwrap_or_default(),
            None => "[]".to_owned(),
        }
    }

    pub fn start_focus(&mut self, power_id: &str, focus_id: u32) -> bool {
        match self.trees.get_mut(power_id) {
            Some(tree) => tree.start_focus(FocusId(focus_id)),
            None => false,
        }
    }

    pub fn advance_focus(&mut self, power_id: &str, days: u32) -> Option<Vec<FocusEffect>> {
        match self.trees.get_mut(power_id) {
            Some(tree) => tree.advance(days),
            None => None,
        }
    }
}

// ── Helper macro to reduce boilerplate ──

macro_rules! focus {
    ($id:expr, $name:expr, $desc:expr, $power:expr, $days:expr,
     [$($pre:expr),*], [$($eff:expr),*], $x:expr, $y:expr, $icon:expr, $cat:expr) => {
        Focus {
            id: FocusId($id),
            name: $name.into(),
            description: $desc.into(),
            power: $power.clone(),
            cost_days: $days,
            prerequisites: vec![$(FocusId($pre)),*],
            effects: vec![$($eff),*],
            x: $x, y: $y,
            icon: $icon.into(),
            category: $cat.into(),
        }
    };
}

// ── France Focus Tree ──

fn build_france_tree() -> FocusTree {
    let p = PowerId::from("FRA");
    let mut tree = FocusTree::new(p.clone());

    // Military branch
    tree.add_focus(focus!(1, "Grande Armée Reform",
        "Reorganize the French army into a modern instrument of war.",
        p, 70, [], [FocusEffect::AttackBonus(10)], 0, 0, "⚔️", "military"));
    tree.add_focus(focus!(2, "Corps System",
        "Adopt the corps d'armée system for independent combined-arms formations.",
        p, 70, [1], [FocusEffect::SupplyRangeBonus(25)], -1, 1, "⚔️", "military"));
    tree.add_focus(focus!(3, "Imperial Guard",
        "Expand the elite Imperial Guard into a full corps of veterans.",
        p, 140, [2], [FocusEffect::UnlockUnit("Imperial Guard".into())], -1, 2, "👑", "military"));
    tree.add_focus(focus!(4, "Light Infantry Doctrine",
        "Train voltigeur and chasseur companies in skirmish warfare.",
        p, 70, [1], [FocusEffect::DefenseBonus(10)], 1, 1, "⚔️", "military"));

    // Economic branch
    tree.add_focus(focus!(5, "Continental System",
        "Impose an economic blockade on Britain across Europe.",
        p, 70, [], [FocusEffect::TreasuryBonus(200)], 3, 0, "💰", "economic"));
    tree.add_focus(focus!(6, "Economic Dominance",
        "Establish French commercial supremacy over continental markets.",
        p, 140, [5], [FocusEffect::TreasuryBonus(400)], 3, 1, "💰", "economic"));
    tree.add_focus(focus!(7, "European Hegemony",
        "France dominates the continent — all nations bend the knee.",
        p, 210, [6], [FocusEffect::DiplomaticInfluence("ALL".into(), 50), FocusEffect::TreasuryBonus(500)],
        3, 2, "👑", "economic"));
    tree.add_focus(focus!(8, "Blockade Britain",
        "Enforce the Continental System and strangle British trade.",
        p, 105, [5], [FocusEffect::NavalBonus(10), FocusEffect::DiplomaticInfluence("GBR".into(), -30)],
        5, 1, "⚓", "economic"));

    // Political branch
    tree.add_focus(focus!(9, "Napoleonic Code",
        "Codify civil law across the Empire, modernizing governance.",
        p, 70, [], [FocusEffect::ResearchBonus(15)], 7, 0, "🏛️", "political"));
    tree.add_focus(focus!(10, "Administrative Reform",
        "Rationalize the prefectural system and tax collection.",
        p, 70, [9], [FocusEffect::TreasuryBonus(150)], 7, 1, "🏛️", "political"));
    tree.add_focus(focus!(11, "Centralized State",
        "Complete centralization of the French administrative apparatus.",
        p, 140, [10], [FocusEffect::ResearchBonus(20), FocusEffect::TreasuryBonus(250)],
        7, 2, "🏛️", "political"));

    // Manpower branch
    tree.add_focus(focus!(12, "Conscription Levée",
        "Institute mass conscription under the Jourdan Law.",
        p, 35, [], [FocusEffect::ManpowerBonus(50_000)], 10, 0, "⚔️", "military"));
    tree.add_focus(focus!(13, "Mass Mobilization",
        "Call up the reserves and expand the training depots.",
        p, 70, [12], [FocusEffect::ManpowerBonus(100_000)], 10, 1, "⚔️", "military"));
    tree.add_focus(focus!(14, "Grand Armée 600K",
        "The Grande Armée reaches its full strength of 600,000 men.",
        p, 140, [13], [FocusEffect::ManpowerBonus(200_000), FocusEffect::AttackBonus(5)],
        10, 2, "👑", "military"));

    tree
}

// ── Britain Focus Tree ──

fn build_britain_tree() -> FocusTree {
    let p = PowerId::from("GBR");
    let mut tree = FocusTree::new(p.clone());

    // Naval branch
    tree.add_focus(focus!(101, "Naval Supremacy",
        "The Royal Navy must command the seas absolutely.",
        p, 70, [], [FocusEffect::NavalBonus(20)], 0, 0, "⚓", "naval"));
    tree.add_focus(focus!(102, "Ship of the Line Program",
        "Expand the fleet with 74-gun ships of the line.",
        p, 140, [101], [FocusEffect::NavalBonus(30), FocusEffect::UnlockUnit("First Rate Ship".into())],
        -1, 1, "⚓", "naval"));
    tree.add_focus(focus!(103, "Rule Britannia",
        "Britannia rules the waves — total naval dominance achieved.",
        p, 210, [102], [FocusEffect::NavalBonus(50), FocusEffect::DiplomaticInfluence("ALL".into(), 25)],
        -1, 2, "👑", "naval"));
    tree.add_focus(focus!(104, "Blockade France",
        "Enforce a naval blockade of French and allied ports.",
        p, 105, [101], [FocusEffect::NavalBonus(15), FocusEffect::DiplomaticInfluence("FRA".into(), -30)],
        1, 1, "⚓", "naval"));

    // Diplomatic branch
    tree.add_focus(focus!(105, "Coalition Building",
        "Forge alliances against France on the continent.",
        p, 70, [], [FocusEffect::DiplomaticInfluence("ALL".into(), 15)], 3, 0, "🏛️", "political"));
    tree.add_focus(focus!(106, "Subsidize Allies",
        "Use British gold to fund continental armies against Napoleon.",
        p, 105, [105], [
            FocusEffect::DiplomaticInfluence("AUS".into(), 25),
            FocusEffect::DiplomaticInfluence("PRU".into(), 25),
            FocusEffect::DiplomaticInfluence("RUS".into(), 25)
        ], 3, 1, "💰", "political"));
    tree.add_focus(focus!(107, "Grand Coalition",
        "The Third Coalition is formed — all Europe stands against France.",
        p, 140, [106], [FocusEffect::DiplomaticInfluence("ALL".into(), 40), FocusEffect::AttackBonus(5)],
        3, 2, "👑", "political"));

    // Industrial branch
    tree.add_focus(focus!(108, "Industrial Revolution",
        "Harness the power of industry to fuel the war effort.",
        p, 70, [], [FocusEffect::TreasuryBonus(300)], 6, 0, "💰", "economic"));
    tree.add_focus(focus!(109, "Steam Power",
        "Apply Watt's steam engine to manufacturing and transport.",
        p, 140, [108], [FocusEffect::ResearchBonus(25), FocusEffect::TreasuryBonus(200)],
        6, 1, "💰", "economic"));
    tree.add_focus(focus!(110, "Factory System",
        "Industrialize arms production with the factory system.",
        p, 140, [109], [FocusEffect::TreasuryBonus(400), FocusEffect::ManpowerBonus(30_000)],
        6, 2, "💰", "economic"));

    // Army branch
    tree.add_focus(focus!(111, "Wellington's Army",
        "Reform the British Army under Sir Arthur Wellesley.",
        p, 105, [], [FocusEffect::DefenseBonus(15), FocusEffect::AttackBonus(10)],
        9, 0, "⚔️", "military"));
    tree.add_focus(focus!(112, "Peninsula Campaign",
        "Launch a major campaign in Iberia to bleed France dry.",
        p, 140, [111], [FocusEffect::AttackBonus(15), FocusEffect::DiplomaticInfluence("SPA".into(), 30)],
        9, 1, "⚔️", "military"));

    tree
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn france_tree_has_14_focuses() {
        let tree = build_france_tree();
        assert_eq!(tree.focuses.len(), 14);
    }

    #[test]
    fn britain_tree_has_12_focuses() {
        let tree = build_britain_tree();
        assert_eq!(tree.focuses.len(), 12);
    }

    #[test]
    fn root_focuses_are_available() {
        let tree = build_france_tree();
        // Grande Armée Reform has no prereqs
        assert!(tree.is_available(FocusId(1)));
        // Corps System requires Grande Armée Reform
        assert!(!tree.is_available(FocusId(2)));
    }

    #[test]
    fn start_focus_works() {
        let mut tree = build_france_tree();
        assert!(tree.start_focus(FocusId(1)));
        assert_eq!(tree.in_progress, Some((FocusId(1), 70)));
    }

    #[test]
    fn cannot_start_two_focuses() {
        let mut tree = build_france_tree();
        assert!(tree.start_focus(FocusId(1)));
        assert!(!tree.start_focus(FocusId(5)));
    }

    #[test]
    fn cannot_start_locked_focus() {
        let mut tree = build_france_tree();
        assert!(!tree.start_focus(FocusId(2)));
    }

    #[test]
    fn advance_completes_focus() {
        let mut tree = build_france_tree();
        tree.start_focus(FocusId(1));
        let effects = tree.advance(70);
        assert!(effects.is_some());
        assert!(tree.completed.contains(&FocusId(1)));
        assert!(tree.in_progress.is_none());
    }

    #[test]
    fn advance_partial_progress() {
        let mut tree = build_france_tree();
        tree.start_focus(FocusId(1));
        let effects = tree.advance(30);
        assert!(effects.is_none());
        assert_eq!(tree.in_progress, Some((FocusId(1), 40)));
    }

    #[test]
    fn completing_prereq_unlocks_next() {
        let mut tree = build_france_tree();
        tree.start_focus(FocusId(1));
        tree.advance(70);
        assert!(tree.is_available(FocusId(2)));
        assert!(tree.is_available(FocusId(4)));
    }

    #[test]
    fn focus_effects_are_correct() {
        let tree = build_france_tree();
        let grande_armee = tree.focuses.get(&FocusId(1)).unwrap();
        assert_eq!(grande_armee.effects, vec![FocusEffect::AttackBonus(10)]);
    }

    #[test]
    fn registry_default_has_two_powers() {
        let reg = FocusRegistry::with_defaults();
        assert!(reg.get_tree("FRA").is_some());
        assert!(reg.get_tree("GBR").is_some());
        assert!(reg.get_tree("AUS").is_none());
    }

    #[test]
    fn registry_start_and_advance() {
        let mut reg = FocusRegistry::with_defaults();
        assert!(reg.start_focus("FRA", 12)); // Conscription (35 days)
        let effects = reg.advance_focus("FRA", 35);
        assert!(effects.is_some());
        let effects = effects.unwrap();
        assert_eq!(effects, vec![FocusEffect::ManpowerBonus(50_000)]);
    }

    #[test]
    fn tree_json_round_trip() {
        let reg = FocusRegistry::with_defaults();
        let json = reg.get_tree_json("FRA");
        assert!(json.contains("Grande"));
        assert!(json.contains("Continental System"));
    }

    #[test]
    fn completed_json_empty_initially() {
        let reg = FocusRegistry::with_defaults();
        let json = reg.get_completed_json("FRA");
        assert_eq!(json, "[]");
    }

    #[test]
    fn completed_focus_not_available() {
        let mut tree = build_france_tree();
        tree.start_focus(FocusId(1));
        tree.advance(70);
        assert!(!tree.is_available(FocusId(1)));
    }

    #[test]
    fn britain_naval_chain() {
        let mut tree = build_britain_tree();
        // Naval Supremacy -> Ship of the Line -> Rule Britannia
        assert!(tree.start_focus(FocusId(101)));
        tree.advance(70);
        assert!(tree.is_available(FocusId(102)));
        assert!(tree.start_focus(FocusId(102)));
        tree.advance(140);
        assert!(tree.is_available(FocusId(103)));
    }

    #[test]
    fn advance_with_no_progress_returns_none() {
        let mut tree = build_france_tree();
        assert!(tree.advance(10).is_none());
    }

    #[test]
    fn unknown_power_returns_null_json() {
        let reg = FocusRegistry::with_defaults();
        assert_eq!(reg.get_tree_json("AUS"), "null");
    }
}
