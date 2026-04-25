//! Scenario root types.  Loaded from `data/scenarios/<id>/scenario.json`.
//!
//! Every numeric field that affects state is an integer.  Anything that
//! would naturally be a fraction (probabilities, percentages) is stored
//! as fixed-point with a documented scale; this is a hard rule from
//! PROMPT.md §2.2.

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::ids::{AreaId, CorpsId, FleetId, LeaderId, MinorId, PowerId, SeaZoneId};
use crate::tables::Maybe;

/// Schema version for the scenario format itself.  Bump on any
/// breaking change; migrations live in `core-schema/migrations/`.
pub const SCHEMA_VERSION: u32 = 1;

/// A complete scenario — the immutable starting state of a campaign.
///
/// This type is the root of `data/scenarios/<id>/scenario.json`.
/// Per §5.3, every persisted root carries `schema_version`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Scenario {
    pub schema_version: u32,
    /// Bumped whenever a tables file changes semantics (PROMPT.md §6.4).
    pub rules_version: u32,
    /// Stable identifier for this scenario, lowercase snake-case.
    pub scenario_id: String,
    /// Display name (English source; localization handles other locales).
    pub name: String,
    /// Inclusive starting (year, month).  `month` is 1..=12.
    pub start: GameDate,
    /// Inclusive ending (year, month).
    pub end: GameDate,
    /// Hard release-blocker flag.  Set `true` while any PLACEHOLDER
    /// values remain (PROMPT.md §6.1).
    pub unplayable_in_release: bool,
    /// Toggleable rules.  All optional rules default to `false`.
    #[serde(default)]
    pub features: Features,
    /// Movement rules.  Mostly designer-authored numerics; placeholders
    /// permitted per PROMPT.md §6.1.
    #[serde(default)]
    pub movement_rules: MovementRules,
    /// Logical turn index since `start` (turn 0 is the start month).
    /// PROMPT.md §2.2 forbids wall-clock; this is the only time that
    /// matters in the simulation.
    #[serde(default)]
    pub current_turn: u32,
    /// Live economic and political state per power.  Initialized from
    /// `PowerSetup.starting_*` at load time when absent (`#[serde(default)]`
    /// makes the JSON omit-compatible).
    #[serde(default)]
    pub power_state: BTreeMap<PowerId, PowerState>,
    /// Pending production deliveries.
    #[serde(default)]
    pub production_queue: Vec<ProductionItem>,
    /// Pending manpower returns from past combat losses (§7.9).
    #[serde(default)]
    pub replacement_queue: Vec<ReplacementItem>,
    /// Pending subsidy transfers, queued by `SubsidyOrder`.
    #[serde(default)]
    pub subsidy_queue: Vec<PendingSubsidy>,
    /// Major powers, keyed by stable ID.  `BTreeMap` for deterministic
    /// iteration (§2.2).
    pub powers: BTreeMap<PowerId, PowerSetup>,
    /// Minor countries.
    pub minors: BTreeMap<MinorId, MinorSetup>,
    /// Leaders, keyed by ID; lookups use the same map regardless of
    /// whether the leader is currently in play.
    pub leaders: BTreeMap<LeaderId, Leader>,
    /// Land areas on the strategic map.
    pub areas: BTreeMap<AreaId, Area>,
    /// Sea zones for naval movement.
    pub sea_zones: BTreeMap<SeaZoneId, SeaZone>,
    /// Starting corps.
    pub corps: BTreeMap<CorpsId, Corps>,
    /// Starting fleets.
    pub fleets: BTreeMap<FleetId, Fleet>,
    /// Starting diplomatic relations.  Keyed by the lexicographically
    /// smaller power; pair semantics are symmetric.
    pub diplomacy: BTreeMap<DiplomaticPairKey, DiplomaticState>,
    /// Adjacency between land areas (undirected; both directions stored
    /// for query simplicity, validated symmetric at load).
    pub adjacency: Vec<AreaAdjacency>,
    /// Adjacency between land areas and sea zones (for ports).
    pub coast_links: Vec<CoastLink>,
    /// Sea-zone adjacency (undirected).
    pub sea_adjacency: Vec<SeaAdjacency>,
}

/// `(year, month)` pair — month is 1..=12.  No wall-clock time enters
/// the simulation core, only this logical date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct GameDate {
    pub year: i32,
    pub month: u8,
}

impl GameDate {
    pub fn new(year: i32, month: u8) -> Self {
        Self { year, month }
    }
}

/// Optional rules toggleable per scenario.  Per PROMPT.md §23.7 the
/// Continental System is out of v1.0; the flag exists so the eventual
/// data pack has a place to land.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Features {
    #[serde(default)]
    pub continental_system: bool,
    /// Named-events system (§7.8).  In for v1.0 per ADR 0001.
    #[serde(default)]
    pub named_events: bool,
}

/// Per-power live state (Phase 3).  The `starting_*` fields on
/// `PowerSetup` are immutable scenario authoring; this struct holds
/// the values that mutate during play.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PowerState {
    pub treasury: i64,
    pub manpower: i32,
    pub prestige: i32,
    pub tax_policy: TaxPolicy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaxPolicy {
    Low,
    #[default]
    Standard,
    Heavy,
}

/// Pending unit production, materializes at `eta_turn`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductionItem {
    pub owner: PowerId,
    pub area: AreaId,
    pub kind: ProductionKind,
    pub eta_turn: u32,
    /// Pre-paid composition for a corps; ignored for other kinds.
    #[serde(default)]
    pub corps_composition: Option<CorpsComposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProductionKind {
    Corps,
    Fleet,
    Depot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CorpsComposition {
    pub infantry_sp: i32,
    pub cavalry_sp: i32,
    pub artillery_sp: i32,
}

/// Pending manpower replacements (§7.9), queued from prior combat
/// losses and arriving `eta_turn` later.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplacementItem {
    pub owner: PowerId,
    pub sp_amount: i32,
    pub eta_turn: u32,
}

/// Pending subsidy transfer awaiting the next economic phase.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PendingSubsidy {
    pub from: PowerId,
    pub to: PowerId,
    pub amount: i64,
}

/// Movement rules (Phase 2).  Every numeric is `Maybe<i32>` to permit
/// PLACEHOLDER values until a designer authors them.  Defaults are
/// chosen so tests can run with explicit `Maybe::Value(_)` overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct MovementRules {
    /// Maximum corps simultaneously in one area.  Designer-authored.
    pub max_corps_per_area: Maybe<i32>,
    /// Standard per-turn movement budget in hop count.  Designer-authored.
    pub movement_hops_per_turn: Maybe<i32>,
    /// Forced-march extra-hop allowance (typically `1`).
    pub forced_march_extra_hops: Maybe<i32>,
    /// Q4 morale loss applied on every forced march, regardless of die
    /// outcome.  PROMPT.md §6.2 morale.json.
    pub forced_march_morale_loss_q4: Maybe<i32>,
}

// ─── Powers ────────────────────────────────────────────────────────────

/// Initial setup for a major power.
///
/// `starting_treasury` and `income` are integers in "francs" (the
/// scenario's accounting unit; one unit per `Money` step).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PowerSetup {
    pub display_name: String,
    pub house: String,
    pub ruler: LeaderId,
    pub capital: AreaId,
    pub starting_treasury: i64,
    pub starting_manpower: i32,
    pub starting_pp: i32,
    pub max_corps: u8,
    pub max_depots: u8,
    pub mobilization_areas: Vec<AreaId>,
    /// Heraldic display color (sRGB hex, e.g. `#2a3a6a`); UI only.
    pub color_hex: String,
}

// ─── Minors ────────────────────────────────────────────────────────────

/// Initial setup for a minor country.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MinorSetup {
    pub display_name: String,
    pub home_areas: Vec<AreaId>,
    pub initial_relationship: MinorRelationship,
    /// If `Allied` / `Feudal` / `Conquered`, the major they are tied to.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub patron: Option<PowerId>,
    pub starting_force_level: i32,
}

/// State machine for a minor's relationship with the powers (§7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MinorRelationship {
    IndependentFree,
    AlliedFree,
    Feudal,
    Conquered,
    InRevolt,
}

// ─── Leaders ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Leader {
    pub display_name: String,
    pub strategic: u8,
    pub tactical: u8,
    pub initiative: u8,
    /// `true` if this leader can command an entire army (multi-corps);
    /// `false` for corps commanders only.
    #[serde(default)]
    pub army_commander: bool,
    /// Birth (year, month).  Used for age-of-death rolls (§7.10).
    pub born: GameDate,
}

// ─── Areas / Sea zones ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Area {
    pub display_name: String,
    pub owner: Owner,
    pub terrain: Terrain,
    /// Fortress level 0..=5; 0 = no fortifications.
    pub fort_level: u8,
    /// Monthly economic yield (francs).  Designer-authored; may be a
    /// PLACEHOLDER per §6.1 until filled.
    pub money_yield: Maybe<i32>,
    /// Monthly manpower yield (SP / 12, integer).  Designer-authored.
    pub manpower_yield: Maybe<i32>,
    #[serde(default)]
    pub capital_of: Option<PowerId>,
    #[serde(default)]
    pub port: bool,
    /// Whether this area is currently under naval blockade.  Blockaded areas
    /// yield no income during the economic phase (PROMPT.md §16.4).
    #[serde(default)]
    pub blockaded: bool,
    /// Strategic map coords (1400×900 viewBox), UI only.
    pub map_x: i32,
    pub map_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeaZone {
    pub display_name: String,
    /// Optional bounding-box centroid for UI.
    pub map_x: i32,
    pub map_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Terrain {
    Open,
    Forest,
    Mountain,
    Marsh,
    Urban,
}

/// Owner of an area or unit — either a major, a minor, or unowned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Owner {
    Power(PowerSlot),
    Minor(MinorSlot),
    Unowned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PowerSlot {
    pub power: PowerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinorSlot {
    pub minor: MinorId,
}

// ─── Corps / Fleets ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Corps {
    pub display_name: String,
    pub owner: PowerId,
    pub area: AreaId,
    pub infantry_sp: i32,
    pub cavalry_sp: i32,
    pub artillery_sp: i32,
    /// Morale 0..=10000 (fixed-point, denominator 10000).
    pub morale_q4: i32,
    pub supplied: bool,
    #[serde(default)]
    pub leader: Option<LeaderId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Fleet {
    pub display_name: String,
    pub owner: PowerId,
    pub at_port: Option<AreaId>,
    pub at_sea: Option<SeaZoneId>,
    pub ships_of_the_line: i32,
    pub frigates: i32,
    pub transports: i32,
    pub morale_q4: i32,
    #[serde(default)]
    pub admiral: Option<LeaderId>,
}

// ─── Diplomacy ─────────────────────────────────────────────────────────

/// Stable, ordered key for a pair of powers.  Always stores `(lo, hi)`
/// in lexicographic order so a single entry expresses a symmetric
/// relation deterministically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct DiplomaticPairKey(pub PowerId, pub PowerId);

impl DiplomaticPairKey {
    pub fn new(a: PowerId, b: PowerId) -> Self {
        if a <= b { Self(a, b) } else { Self(b, a) }
    }
}

// Serialize as "<lo>:<hi>" so it's a JSON object key.
impl Serialize for DiplomaticPairKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let combined = format!("{}:{}", self.0, self.1);
        s.serialize_str(&combined)
    }
}

impl<'de> Deserialize<'de> for DiplomaticPairKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        let (a, b) = raw
            .split_once(':')
            .ok_or_else(|| serde::de::Error::custom(format!("invalid pair key `{raw}`")))?;
        Ok(Self::new(PowerId::from(a), PowerId::from(b)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiplomaticState {
    War,
    Unfriendly,
    Neutral,
    Friendly,
    Allied,
}

// ─── Adjacency ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AreaAdjacency {
    pub from: AreaId,
    pub to: AreaId,
    /// Land-link cost.  Designer-authored per movement table.
    pub cost: Maybe<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoastLink {
    pub area: AreaId,
    pub sea: SeaZoneId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeaAdjacency {
    pub from: SeaZoneId,
    pub to: SeaZoneId,
}

// ─── Misc ──────────────────────────────────────────────────────────────

/// A handy, schema-described `IndexMap` re-export for places where
/// insertion order matters (e.g. authored ordering of UI hints).
pub type Ordered<K, V> = IndexMap<K, V>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::{canonical_hash, to_canonical_string};

    fn empty_scenario() -> Scenario {
        Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "smoke".into(),
            name: "Smoke Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules::default(),
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
        }
    }

    #[test]
    fn empty_scenario_round_trips_canonically() {
        let s1 = empty_scenario();
        let canon1 = to_canonical_string(&s1).unwrap();
        let s2: Scenario = serde_json::from_str(&canon1).unwrap();
        let canon2 = to_canonical_string(&s2).unwrap();
        assert_eq!(canon1, canon2);
        assert_eq!(canonical_hash(&s1).unwrap(), canonical_hash(&s2).unwrap());
    }

    #[test]
    fn pair_key_orders_canonically() {
        let a = DiplomaticPairKey::new(PowerId::from("RUS"), PowerId::from("FRA"));
        let b = DiplomaticPairKey::new(PowerId::from("FRA"), PowerId::from("RUS"));
        assert_eq!(a, b);
        let s = serde_json::to_string(&a).unwrap();
        assert_eq!(s, "\"FRA:RUS\"");
    }

    #[test]
    fn diplomatic_state_round_trip() {
        let v = DiplomaticState::Allied;
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, "\"ALLIED\"");
        let back: DiplomaticState = serde_json::from_str(&s).unwrap();
        assert_eq!(v, back);
    }
}
