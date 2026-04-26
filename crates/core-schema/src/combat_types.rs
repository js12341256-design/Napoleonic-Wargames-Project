//! Combat outcome and casualty types for Phase 4.
//!
//! Separated from events.rs to keep files focused.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The strategic result of a resolved battle (PROMPT.md §16.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BattleOutcome {
    AttackerRepulsed,
    DefenderRetreats,
    DefenderRouted,
    MutualWithdrawal,
}

/// Whether the leader survived, was wounded, or was killed (PROMPT.md §16.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaderCasualtyKind {
    Unharmed,
    Wounded,
    Killed,
}
