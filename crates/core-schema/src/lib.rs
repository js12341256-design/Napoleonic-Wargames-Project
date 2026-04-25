//! Grand Campaign 1805 — data schemas.
//!
//! All persisted state lives here as typed Rust.  Three families of types:
//!
//! - **IDs** ([`ids`]) — stable, namespaced string identifiers per
//!   [`PROMPT.md §5.1`](../../docs/PROMPT.md).
//! - **Scenario** ([`scenario`]) — the immutable, designer-authored game
//!   setup loaded from `data/scenarios/<id>/`.
//! - **Tables** ([`tables`]) — the rules numerical values from
//!   `data/tables/`, with explicit [`tables::Placeholder`] support per
//!   [`PROMPT.md §6.1`](../../docs/PROMPT.md).
//!
//! Every persisted type implements canonical JSON via the
//! [`canonical`] module — this is the only serialization permitted for
//! hashing or save files (PROMPT.md §5.2).
//!
//! # Phase 1 status
//!
//! Types defined; placeholder-friendly.  Behavior (validators,
//! resolvers) lives in `gc1805-core` and other crates.
#![forbid(unsafe_code)]

pub mod canonical;
pub mod events;
pub mod ids;
pub mod scenario;
pub mod supply_types;
pub mod tables;

pub use canonical::{canonical_hash, to_canonical_string, CanonicalJsonError};
pub use events::Event;
pub use ids::{AreaId, CorpsId, FleetId, LeaderId, MinorId, PowerId, SeaZoneId};
pub use scenario::{Scenario, SCHEMA_VERSION};
pub use supply_types::SupplyState;

/// Errors raised by schema validation.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("schema_version {found} not supported (oldest supported: {min}, newest: {max})")]
    SchemaVersion { found: u32, min: u32, max: u32 },

    #[error("placeholder at {path}: scenarios with PLACEHOLDER values cannot ship in release")]
    PlaceholderPresent { path: String },

    #[error("missing reference: {referer} → {referent} (kind: {kind})")]
    DanglingReference {
        referer: String,
        referent: String,
        kind: &'static str,
    },

    #[error("invalid value at {path}: {reason}")]
    InvalidValue { path: String, reason: String },
}
