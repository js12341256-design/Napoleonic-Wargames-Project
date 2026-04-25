# Phase 10 — Full turn loop

Date closed: 2026-04-25
Branch: `phase10-turn-loop`
Gate: `docs/PROMPT.md` §16.10

## Summary

Full turn orchestration landed in `gc1805-core::turn_loop`: start-of-turn
marker, economic order application plus economic resolution, movement order
resolution, combat order resolution, stub supply/political phases, turn
increment, and canonical state hashing.  Schema event vocabulary now includes
`TurnStarted`, `PhaseCompleted`, and `TurnCompleted`.

24 hand-written tests cover turn counting, event emission, hash shape,
determinism, empty-input completion, movement/combat/economic execution, phase
ordering, placeholder-table tolerance, and rejection paths.

Workspace brought back to clean `fmt`, `clippy -D warnings`, and `test`.
Schemas regenerated.

## Gate evidence

| §16.10 requirement | Status |
|---|---|
| Full turn orchestrator exists | ✅ `crates/core/src/turn_loop.rs::run_turn` |
| Economic, movement, combat phases invoked in fixed order | ✅ `run_turn` resolves them sequentially and emits `PhaseCompleted` after each |
| Supply and political phases stubbed but visible | ✅ Completion events emitted for both |
| Canonical end-of-turn state hash computed | ✅ `TurnCompleted.state_hash` + `TurnOutput.state_hash` |
| Determinism covered by tests | ✅ same seed / same scenario => same hash and matching events |
| 20+ hand-written tests | ✅ 24 tests in `turn_loop.rs::tests` |

## What was built

### `gc1805-core-schema`

- `events.rs` — added:
  - `TurnStarted { turn }`
  - `PhaseCompleted { turn, phase_name }`
  - `TurnCompleted { turn, state_hash }`

### `gc1805-core`

- `turn_loop.rs` — new module containing:
  - `AllTables`
  - `TurnInput`
  - `TurnOutput`
  - `run_turn(&mut Scenario, &AllTables, TurnInput, u64) -> TurnOutput`
- `lib.rs` — exports `pub mod turn_loop;`

### Documentation

- `docs/rules/turn_loop.md` — six-stage turn reference.
- `docs/phase-reports/phase-10.md` — this file.

### Schemas

- `data/schemas/` regenerated so the new event variants are reflected in the
  published schema output.

## ADRs added

None.

## Adjudications added

None.

## Open questions

No new blocking questions introduced by the turn orchestrator.  Existing
placeholder-driven questions from prior phases still carry forward for supply,
attrition, and fuller political systems.

## Known defects and caveats

- Supply and political are explicit stubs for now: phase markers exist, but
  no state mutation happens yet.
- Turn hashing uses the canonical state API already present in
  `core-schema::canonical`; future phases may choose to hash event log + state
  together, but the current Phase 10 contract hashes the post-turn scenario.
- Economic orders are applied in-order before `resolve_economic_phase`, which
  makes `SetTaxPolicy` and other queued economic actions visible inside the
  same orchestrated turn.

## Next phase

Wire real supply / attrition and political resolution into the existing stub
slots without changing the outer turn contract.
