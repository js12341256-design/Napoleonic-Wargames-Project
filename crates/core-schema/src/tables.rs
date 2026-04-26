//! Rules tables.  Designer-authored values in `data/tables/*.json`.
//!
//! PROMPT.md §6.1: Claude Code never invents values.  Missing values
//! load as [`Maybe::Placeholder`] and surface as warnings; the scenario
//! is then `unplayable_in_release: true`.
//!
//! Phase 1 status: schemas only.  Validators and consumers land in their
//! respective rules-subsystem phases.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Either a designer-authored value of type `T`, or an explicit
/// PLACEHOLDER marker.  A scenario carrying any `Placeholder` cannot
/// ship in release per §6.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Maybe<T> {
    Value(T),
    Placeholder(PlaceholderMarker),
}

/// A literal `{"_placeholder": true}` JSON object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PlaceholderMarker {
    #[serde(rename = "_placeholder")]
    pub _placeholder: bool,
}

impl PlaceholderMarker {
    pub fn new() -> Self {
        Self { _placeholder: true }
    }
}

impl Default for PlaceholderMarker {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Maybe<T> {
    pub fn placeholder() -> Self {
        Self::Placeholder(PlaceholderMarker::new())
    }
    pub fn is_placeholder(&self) -> bool {
        matches!(self, Self::Placeholder(_))
    }
    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Value(v) => Some(v),
            Self::Placeholder(_) => None,
        }
    }
}

impl<T> Default for Maybe<T> {
    fn default() -> Self {
        Self::placeholder()
    }
}

// ─── Combat (PROMPT.md §6.2) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CombatTable {
    pub schema_version: u32,
    pub rules_version: Option<String>,
    pub die_faces: u8,
    pub die_count: Option<u8>,
    pub ratio_buckets: Vec<RatioBucket>,
    pub formations: Vec<Formation>,
    pub formation_matrix: DocumentedMap<FormationEntry>,
    pub terrain_modifiers: DocumentedMap<TerrainModifier>,
    /// Keyed by `ratio_bucket`; each entry has one `CombatResult` per
    /// die face (length must equal `die_faces`).
    #[serde(alias = "results")]
    pub results_table: DocumentedMap<Vec<CombatResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RatioBucket {
    pub id: String,
    pub min_ratio_pct: i32,
    pub max_ratio_pct: i32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Formation {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct DocumentedMap<T> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _doc: Option<JsonValue>,
    #[serde(flatten)]
    pub entries: BTreeMap<String, T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormationEntry {
    pub att_col_shift: i8,
    pub def_morale_shift: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TerrainModifier {
    pub att_col_shift: i8,
    pub extra_def_morale: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CombatResult {
    pub att_sp_loss: i32,
    pub def_sp_loss: i32,
    /// Morale delta in fixed-point quarter-thousandths (`/10000`).
    pub att_morale_delta: i32,
    pub def_morale_delta: i32,
    pub def_retreat_steps: i32,
    pub att_advances: bool,
}

// ─── Attrition ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AttritionTable {
    pub schema_version: u32,
    /// Keyed by `<terrain>_<season>_<supply_state>` →
    /// SP loss per turn.
    pub rows: BTreeMap<String, Maybe<i32>>,
}

// ─── Weather ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WeatherTable {
    pub schema_version: u32,
    pub theaters: Vec<String>,
    /// `month (1..=12)` → theater → distribution over weather kinds.
    /// Each distribution is a map from weather kind to a probability in
    /// fixed-point Q12 (denominator 4096); each row must sum to 4096.
    pub monthly: BTreeMap<u8, BTreeMap<String, BTreeMap<String, Maybe<i32>>>>,
}

// ─── Minor activation ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MinorActivationTable {
    pub schema_version: u32,
    /// Keyed by minor ID → trigger key → outcome distribution.
    pub rows: BTreeMap<String, Maybe<MinorActivationRow>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MinorActivationRow {
    pub trigger: String,
    /// outcome key → Q12 probability.
    pub outcomes: BTreeMap<String, i32>,
}

// ─── PP modifiers ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PpModifiersTable {
    pub schema_version: u32,
    /// Keyed by event name → integer PP delta.
    pub events: BTreeMap<String, Maybe<i32>>,
}

// ─── Leader casualty ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LeaderCasualtyTable {
    pub schema_version: u32,
    /// Battle intensity bucket → outcome distribution (Q12).
    pub by_intensity: BTreeMap<String, BTreeMap<String, Maybe<i32>>>,
}

// ─── Morale ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoraleTable {
    pub schema_version: u32,
    pub retreat_threshold_q4: Maybe<i32>,
    pub rout_threshold_q4: Maybe<i32>,
    pub recovery_per_turn_q4: Maybe<i32>,
}

// ─── Naval combat ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NavalCombatTable {
    pub schema_version: u32,
    pub ratio_buckets: Vec<String>,
    pub die_faces: u8,
    pub results: BTreeMap<String, Vec<Maybe<NavalResult>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NavalResult {
    pub attacker_ship_loss: i32,
    pub defender_ship_loss: i32,
    pub disengage: bool,
}

// ─── Economy ───────────────────────────────────────────────────────────

/// Designer-authored economy values (`data/tables/economy.json`).
/// Every numeric is `Maybe<i32>`; loading with placeholders leaves
/// the scenario `unplayable_in_release`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EconomyTable {
    pub schema_version: u32,

    // Maintenance (Phase 3) ─────────────────────────────────────────
    pub corps_maintenance_per_sp: Maybe<i32>,
    pub fleet_maintenance_per_ship: Maybe<i32>,

    // Tax-policy multipliers, Q4 (denom 10000).  Keyed lexicographically
    // for deterministic iteration; values are `Maybe<i32>`.
    pub tax_policy_multiplier_low_q4: Maybe<i32>,
    pub tax_policy_multiplier_standard_q4: Maybe<i32>,
    pub tax_policy_multiplier_heavy_q4: Maybe<i32>,

    // Production (§7.2) ─────────────────────────────────────────────
    pub corps_build_cost_money: Maybe<i32>,
    pub corps_build_cost_manpower: Maybe<i32>,
    pub corps_production_lag_turns: Maybe<i32>,
    pub corps_minimum_sp: Maybe<i32>,
    pub new_corps_morale_q4: Maybe<i32>,

    pub fleet_build_cost_money: Maybe<i32>,
    pub fleet_production_lag_turns: Maybe<i32>,

    // Depots (§7.3 — Phase 5 wires this in).
    pub depot_build_cost: Maybe<i32>,
    pub max_depots_default: Maybe<i32>,

    // Replacement queue (§7.9) ──────────────────────────────────────
    /// Q12 fraction of last turn's combat losses returned per turn.
    pub manpower_recovery_q12: Maybe<i32>,
    /// How many turns after a loss a replacement arrives.
    pub manpower_recovery_lag_turns: Maybe<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_round_trip() {
        let m: Maybe<i32> = Maybe::placeholder();
        let s = serde_json::to_string(&m).unwrap();
        assert_eq!(s, r#"{"_placeholder":true}"#);
        let back: Maybe<i32> = serde_json::from_str(&s).unwrap();
        assert!(back.is_placeholder());
    }

    #[test]
    fn value_round_trip() {
        let m: Maybe<i32> = Maybe::Value(42);
        let s = serde_json::to_string(&m).unwrap();
        assert_eq!(s, "42");
        let back: Maybe<i32> = serde_json::from_str(&s).unwrap();
        assert_eq!(back.get(), Some(&42));
    }
}
