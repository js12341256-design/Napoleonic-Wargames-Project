# Phase 18 — Replay

Date completed: 2026-04-25
Branch: `phase18-replay`
Gate: `docs/PROMPT.md` §16.18

## Summary

Implemented replay support as an event-log fold over an initial
`Scenario`, with seek-to-turn reconstruction, per-turn integrity hashes,
JSON save/load helpers, and 16 hand-written unit tests in
`crates/core/src/replay.rs`.

Replay is intentionally **read-only** in this phase: it reconstructs the
subset of state transitions explicitly encoded in replay-relevant events
rather than calling the full live resolvers.

## What was built

### `docs/rules/replay.md`

New canonical rules reference covering:

- replay as `initial_scenario + Vec<Event>` fold
- replay file format
- seek-to-turn semantics
- integrity verification by re-deriving hashes
- Phase 18 constraints and current no-op event handling

### `gc1805-core-schema`

- Added `Event::TurnCompleted { turn: u32 }` so the event log can mark
  completed turn boundaries for replay seeking and integrity verification.

### `gc1805-core`

- Added `pub mod replay;` to `crates/core/src/lib.rs`
- Added `crates/core/src/replay.rs` with:
  - `ReplayFile`
  - `TurnHash`
  - `ReplayPlayer`
  - `create_replay`
  - `append_events`
  - `seek_to_turn`
  - `verify_integrity`
  - `save_replay`
  - `load_replay`
- Added a read-only event applier for these Phase 18 replay-visible events:
  - `IncomePaid`
  - `MaintenancePaid`
  - `TreasuryInDeficit`
  - `MovementResolved`
  - `TurnCompleted`

## Tests

16 replay tests added:

1. `create_replay_empty_events`
2. `append_events_grows_event_list`
3. `append_events_records_hash`
4. `seek_to_turn_zero_is_initial`
5. `seek_to_turn_applies_income_event`
6. `seek_to_turn_applies_movement_event`
7. `seek_to_turn_unknown_turn_returns_err`
8. `verify_integrity_clean_replay_ok`
9. `verify_integrity_tampered_hash_fails`
10. `save_and_load_round_trip`
11. `load_invalid_json_returns_err`
12. `turn_hashes_count_matches_turns`
13. `seek_intermediate_turn`
14. `replay_deterministic`
15. `seek_applies_maintenance_and_deficit`
16. `verify_integrity_requires_turn_completed_marker`

## Notes / limitations carried forward

- Replay currently treats all non-listed event variants as no-ops.
  That is deliberate for Phase 18 and should be extended in later phases
  when those events need state reconstruction support.
- Integrity verification depends on explicit `TurnCompleted` markers in
  the event stream for every hashed seek target.
- Hashing uses `canonical_hash(&Scenario)`; replay save/load formatting is
  human-readable JSON but does not define the integrity hash.

## Gate status

Implemented and documented. Final gate status depends on repo-wide
fmt/clippy/test passing in CI/local verification.
