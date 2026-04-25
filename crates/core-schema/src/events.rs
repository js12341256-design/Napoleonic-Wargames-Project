//! Event-sourced log entries (PROMPT.md §2.3).
//!
//! The simulation is the fold of ordered events over the initial
//! scenario.  Events come in three families per §2.4:
//!
//! - **Order** — an accepted player or AI submission.
//! - **Resolution** — a deterministic consequence emitted by the
//!   resolver.
//! - **Reveal** — a previously-secret fact becoming public.
//!
//! Phase 2 introduces the movement-related variants.  Future phases
//! grow the enum without changing existing variants — the canonical
//! serialization is forwards-stable as long as serde tags remain.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ids::{AreaId, CorpsId, PowerId};
use crate::scenario::{ProductionKind, TaxPolicy};

/// Top-level event-log entry.  `serde(tag = "kind")` makes the
/// canonical JSON form `{"kind": "MOVEMENT_RESOLVED", ...fields}`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    /// A `MOVE` order resolved successfully.
    MovementResolved(MovementResolved),
    /// A `FORCED_MARCH` order resolved; attrition values come from the
    /// attrition table (Maybe-valued, may be PLACEHOLDER).
    ForcedMarchResolved(ForcedMarchResolved),
    /// An `INTERCEPTION` order was accepted into the order book.  The
    /// fire-time event lands in Phase 10.
    InterceptionQueued(InterceptionQueued),
    /// An order was rejected at validation.
    OrderRejected(OrderRejected),

    // ─── Economy (Phase 3) ───────────────────────────────────────────────
    /// Income collected at the start of the economic phase.
    IncomePaid {
        power: PowerId,
        /// Raw area yield before tax multiplier.
        gross: i64,
        /// Yield after applying `tax_policy` multiplier (integer Q4 division).
        net: i64,
        tax_policy: TaxPolicy,
    },
    /// Upkeep deducted from treasury for all corps and fleets.
    MaintenancePaid {
        power: PowerId,
        corps_cost: i64,
        fleet_cost: i64,
    },
    /// Treasury ran short; clamped at zero for the remainder of this phase.
    TreasuryInDeficit {
        power: PowerId,
        /// How much could not be paid.
        shortfall: i64,
    },
    /// A replacement batch from the manpower queue arrived.
    ReplacementsArrived { owner: PowerId, sp_amount: i32 },
    /// A production item completed and a new unit entered the scenario.
    UnitProduced {
        owner: PowerId,
        area: AreaId,
        unit_kind: ProductionKind,
    },
    /// A pending subsidy was transferred between powers.
    SubsidyTransferred {
        from: PowerId,
        to: PowerId,
        amount: i64,
    },
    /// A power's tax policy was changed by a `SetTaxPolicy` order.
    TaxPolicySet {
        power: PowerId,
        new_policy: TaxPolicy,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct MovementResolved {
    pub corps: CorpsId,
    pub from: AreaId,
    pub to: AreaId,
    /// Number of land hops crossed.  Identical to `path.len() - 1`.
    pub hops: i32,
    /// Path actually taken, inclusive of `from` and `to`.
    pub path: Vec<AreaId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ForcedMarchResolved {
    pub corps: CorpsId,
    pub from: AreaId,
    pub to: AreaId,
    pub hops: i32,
    pub path: Vec<AreaId>,
    /// Q4 morale delta applied by the forced march itself.  Zero if
    /// the rules-table entry is PLACEHOLDER.
    pub morale_loss_q4: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct InterceptionQueued {
    pub corps: CorpsId,
    pub target_area: AreaId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OrderRejected {
    pub reason_code: String,
    pub message: String,
}
