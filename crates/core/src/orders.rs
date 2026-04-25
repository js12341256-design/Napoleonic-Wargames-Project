//! Player and AI order types (Phase 2).
//!
//! Orders are *submissions*; they enter the event log only after
//! validation accepts them.  Rejected orders never enter the log
//! (PROMPT.md §2.4).
//!
//! Phase 2 introduces the movement family.  Other phases extend the
//! `Order` enum without changing existing variants.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use gc1805_core_schema::ids::{AreaId, CorpsId, PowerId};

/// Top-level order submitted by a player or the AI.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Order {
    /// Hold position — explicit no-op, included so all corps appear
    /// in the order book.
    Hold(HoldOrder),
    /// Move along the shortest legal path.  Path-finding is
    /// deterministic (`crates/core/src/map.rs`).
    Move(MoveOrder),
    /// Move with a single extra hop allowance plus attrition.
    ForcedMarch(ForcedMarchOrder),
    /// Standing-order interception; resolution deferred until Phase 10.
    /// See `docs/adjudications.md` adjudication 0001.
    Interception(InterceptionOrder),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct HoldOrder {
    pub submitter: PowerId,
    pub corps: CorpsId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct MoveOrder {
    pub submitter: PowerId,
    pub corps: CorpsId,
    pub to: AreaId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ForcedMarchOrder {
    pub submitter: PowerId,
    pub corps: CorpsId,
    pub to: AreaId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct InterceptionOrder {
    pub submitter: PowerId,
    pub corps: CorpsId,
    pub target_area: AreaId,
    /// Free-form condition string; the conditional-order grammar lands
    /// in Phase 6.  Phase 2 stores the raw text without parsing.
    #[serde(default)]
    pub condition: String,
}

impl Order {
    pub fn submitter(&self) -> &PowerId {
        match self {
            Order::Hold(o) => &o.submitter,
            Order::Move(o) => &o.submitter,
            Order::ForcedMarch(o) => &o.submitter,
            Order::Interception(o) => &o.submitter,
        }
    }

    pub fn corps(&self) -> &CorpsId {
        match self {
            Order::Hold(o) => &o.corps,
            Order::Move(o) => &o.corps,
            Order::ForcedMarch(o) => &o.corps,
            Order::Interception(o) => &o.corps,
        }
    }
}
