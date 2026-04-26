//! Deterministic RNG abstraction.
//!
//! Streams named per PROMPT.md §2.2: `combat`, `minor_activation`,
//! `ai_decision`, `weather`, `misc`.  Adding a new stream must not
//! perturb existing streams; implementation lands in Phase 2+.
#![forbid(unsafe_code)]
