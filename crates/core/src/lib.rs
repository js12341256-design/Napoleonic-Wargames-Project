//! Grand Campaign 1805 — simulation core.
//!
//! This crate is the pure, deterministic simulation library.
//! It is forbidden from touching I/O, wall-clock time, async runtimes,
//! or hash-ordered iteration.  See `docs/PROMPT.md` §2 and §3.1.
//!
//! Phase 0 status: scaffolding only.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]
