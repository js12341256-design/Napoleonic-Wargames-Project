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
use gc1805_core_schema::scenario::{CorpsComposition, TaxPolicy};

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
    /// Set the power's tax policy; takes effect at the next economic
    /// phase (PROMPT.md §8.2 + Phase 3 rules).
    SetTaxPolicy(SetTaxPolicyOrder),
    /// Build a new corps in a mobilization area.
    BuildCorps(BuildCorpsOrder),
    /// Build a new fleet in a port.
    BuildFleet(BuildFleetOrder),
    /// Send money to another power; applied at the next economic phase.
    Subsidize(SubsidyOrder),
    /// Open war against another major power during the diplomatic phase.
    DeclareWar(DeclareWarOrder),
    /// Propose peace terms to a current enemy.
    ProposePeace(ProposePeaceOrder),
    /// Form an alliance with another power.
    FormAlliance(FormAllianceOrder),
    /// End an existing alliance.
    BreakAlliance(BreakAllianceOrder),
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
pub struct SetTaxPolicyOrder {
    pub submitter: PowerId,
    pub policy: TaxPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BuildCorpsOrder {
    pub submitter: PowerId,
    pub area: AreaId,
    pub composition: CorpsComposition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BuildFleetOrder {
    pub submitter: PowerId,
    pub area: AreaId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SubsidyOrder {
    pub submitter: PowerId,
    pub recipient: PowerId,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DeclareWarOrder {
    pub submitter: PowerId,
    pub target: PowerId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProposePeaceOrder {
    pub submitter: PowerId,
    pub target: PowerId,
    pub terms: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct FormAllianceOrder {
    pub submitter: PowerId,
    pub target: PowerId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BreakAllianceOrder {
    pub submitter: PowerId,
    pub target: PowerId,
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
            Order::SetTaxPolicy(o) => &o.submitter,
            Order::BuildCorps(o) => &o.submitter,
            Order::BuildFleet(o) => &o.submitter,
            Order::Subsidize(o) => &o.submitter,
            Order::DeclareWar(o) => &o.submitter,
            Order::ProposePeace(o) => &o.submitter,
            Order::FormAlliance(o) => &o.submitter,
            Order::BreakAlliance(o) => &o.submitter,
        }
    }

    /// Returns the corps targeted by this order, when it has one.
    /// Economic and diplomatic orders return `None`.
    pub fn corps(&self) -> Option<&CorpsId> {
        match self {
            Order::Hold(o) => Some(&o.corps),
            Order::Move(o) => Some(&o.corps),
            Order::ForcedMarch(o) => Some(&o.corps),
            Order::Interception(o) => Some(&o.corps),
            Order::SetTaxPolicy(_)
            | Order::BuildCorps(_)
            | Order::BuildFleet(_)
            | Order::Subsidize(_)
            | Order::DeclareWar(_)
            | Order::ProposePeace(_)
            | Order::FormAlliance(_)
            | Order::BreakAlliance(_) => None,
        }
    }

    /// True if this is a movement-family order (validated by
    /// [`crate::movement::validate_order`]).
    pub fn is_movement(&self) -> bool {
        matches!(
            self,
            Order::Hold(_) | Order::Move(_) | Order::ForcedMarch(_) | Order::Interception(_)
        )
    }
}
