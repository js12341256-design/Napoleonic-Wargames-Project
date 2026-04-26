//! Grand Campaign 1805 — simulation core.
//!
//! Pure, deterministic library.  No I/O outside the explicit
//! [`loader`] module entry points; no wall-clock; no async; no
//! hash-ordered iteration.  See `docs/PROMPT.md` §2 and §3.1.
//!
//! Phase 1 status: scenario loader, projection, integrity checks.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

pub mod clock;
pub mod combat;
pub mod diplomacy;
pub mod division;
pub mod economy;
pub mod frontlines;
pub mod loader;
pub mod map;
pub mod marshals;
pub mod minors;
pub mod mod_loader;
pub mod movement;
pub mod naval;
pub mod orders;
pub mod political;
pub mod projection;
pub mod replay;
pub mod supply;
pub mod turn_loop;
pub mod validate;

pub use loader::{LoadError, LoadReport, load_scenario_str};
pub use map::MapGraph;
pub use movement::{
    MovementPlan, MovementRejection, resolve_order, validate_or_reject, validate_order,
};
pub use orders::Order;
pub use projection::{project, ProjectedScenario};
pub use supply::{resolve_supply_phase, trace_supply, validate_depot_order};
pub use validate::{validate_scenario, IntegrityIssue};

pub use diplomacy::{
    get_diplomatic_state, resolve_diplomatic_phase, set_diplomatic_state, validate_diplomatic_order,
};
pub use economy::{apply_economic_order, resolve_economic_phase, validate_economic_order};
pub use clock::{GameClock, GameSpeed};
pub use division::{BattleTactic, DivisionRegistry, DivisionTemplate};
pub use marshals::{Marshal, MarshalRegistry, MarshalTrait};
pub use frontlines::{FrontLineManager, BattleEvent, BattleResult, FrontLine};

pub use gc1805_core_schema as schema;
