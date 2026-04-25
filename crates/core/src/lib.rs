//! Grand Campaign 1805 — simulation core.
//!
//! Pure, deterministic library.  No I/O outside the explicit
//! [`loader`] module entry points; no wall-clock; no async; no
//! hash-ordered iteration.  See `docs/PROMPT.md` §2 and §3.1.
//!
//! Phase 1 status: scenario loader, projection, integrity checks.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

pub mod economy;
pub mod loader;
pub mod map;
pub mod movement;
pub mod orders;
pub mod projection;
pub mod validate;

pub use loader::{load_scenario_str, LoadError, LoadReport};
pub use map::MapGraph;
pub use movement::{
    resolve_order, validate_or_reject, validate_order, MovementPlan, MovementRejection,
};
pub use orders::Order;
pub use projection::{project, ProjectedScenario};
pub use validate::{validate_scenario, IntegrityIssue};

pub use economy::{apply_economic_order, resolve_economic_phase, validate_economic_order};
pub use gc1805_core_schema as schema;
