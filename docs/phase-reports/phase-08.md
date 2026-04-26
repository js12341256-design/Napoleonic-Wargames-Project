# Phase 8 — Minors

Date completed: 2026-04-25
Branch: `phase8-minors-q6`
Gate: `docs/PROMPT.md` §16.8

## Summary

Integrated the designer-supplied Q6 minors pack into the 1805 scenario,
expanded the scenario from a single Bavaria stub to the full authored minor
roster, added a deterministic minor activation/state module, introduced
minor-related event variants, and added 20+ hand-written tests.

`data/tables/minors.json` now preserves the richer Q6 payload for future
formed-minor and tuning work, while `scenario.json` carries the compact
runtime-ready projection required by the current schema.

## Gate evidence

| §16.8 requirement | Status |
|---|---|
| Full minors roster integrated into 1805 scenario | ✅ `data/scenarios/1805_standard/scenario.json` now includes all Q6 minors |
| Placeholder areas added for missing minor references | ✅ Missing home/capital/fortress areas materialized deterministically |
| Minor activation/state logic implemented | ✅ `crates/core/src/minors.rs` |
| Minor events added | ✅ `MinorActivated`, `MinorRevolt` in `gc1805-core-schema` |
| 20+ hand-written tests | ✅ 24 tests in `crates/core/src/minors.rs` |
| Q6 closed in docs/questions.md | ✅ closed on 2026-04-25 |
| Documentation updated | ✅ `docs/rules/minors.md`, this report, changelog |

## What was built

### Data

- `data/scenarios/1805_standard/scenario.json`
  - replaced the one-entry Bavaria placeholder with the full Q6 roster
  - mapped Q6 starting states into the current `MinorSetup` schema
  - added placeholder areas for referenced minor territories not already in
    the scenario
- `data/tables/minors.json`
  - stores the richer Q6 source payload
  - includes deterministic placeholder activation rows for every minor

### `gc1805-core-schema`

- `events.rs`
  - added `MinorActivated { minor, new_status, patron }`
  - added `MinorRevolt { minor, area }`

### `gc1805-core`

- `minors.rs`
  - `MinorStatus` runtime enum
  - `activate_minor`
  - `validate_minor_control`
  - helpers for mapping weighted activation rows and placeholder fallback
- `lib.rs`
  - exports `pub mod minors;`

## Notes and caveats

- The supplied Q6 data pack is richer than the current `Scenario::MinorSetup`
  schema. Phase 8 therefore preserves the full source payload separately in
  `data/tables/minors.json` instead of widening the persisted scenario root.
- Placeholder area coordinates are `0,0`. They satisfy schema and structural
  validation, but map-authoring polish still belongs to later passes.
- Some Q6 statuses such as `NONEXISTENT_AT_START` cannot be represented 1:1
  by the current compact scenario schema, so the richer source data remains
  authoritative for future formed-minor/event work.

## Open follow-up

- Later phases may want a dedicated persisted minor-state structure rather
  than overloading `MinorSetup.initial_relationship` for mutable runtime
  transitions.
- Formed-minor creation and exact activation-rule tuning still depend on the
  richer Q6 data and future diplomatic/event work.
