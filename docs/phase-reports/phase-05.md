# Phase 5 — Supply

Date closed: 2026-04-25
Branch: `phase5-supply`
Gate: `docs/PROMPT.md` §16.5

## Summary

Supply tracing now exists as a deterministic core subsystem: corps can be
classified as `InSupply`, `Foraging`, or `OutOfSupply`; the supply phase emits
explicit events; out-of-supply attrition reads authored data without inventing
numbers; and depot establishment has typed order validation.

128 tests passing total after Phase 5 (92 prior + 36 supply tests).
Workspace clean under fmt, clippy, and full workspace test.

## Gate evidence

| §16.5 requirement | Status |
|---|---|
| Supply state enum implemented | ✅ `gc1805_core_schema::SupplyState` with canonical serde/schema support |
| Supply trace from corps to capital/depot | ✅ `gc1805_core::trace_supply` with deterministic BFS |
| Enemy ZoC interrupts the trace | ✅ Intermediate-area ZoC blocking covered by dedicated tests |
| Foraging when cut off but local yield exists | ✅ `SupplyState::Foraging` plus tests for value vs placeholder yield |
| Attrition only for truly out-of-supply corps | ✅ `resolve_supply_phase` applies attrition only to `OutOfSupply` |
| No invented attrition numbers | ✅ Placeholder rows skip loss; value rows apply exact authored loss |
| Depot order validation exists | ✅ `EstablishDepotOrder` + `validate_depot_order` |
| 30+ hand-written tests | ✅ 36 tests in `crates/core/src/supply.rs::tests` |

## What was built

### `gc1805-core-schema`

- New `supply_types.rs` with `SupplyState { InSupply, Foraging, OutOfSupply }`.
- `lib.rs` re-exports `SupplyState`.
- `events.rs` adds:
  - `SupplyTraced { corps, supply_state }`
  - `AttritionApplied { corps, sp_loss, reason }`

### `gc1805-core`

- `orders.rs`
  - Added `Order::EstablishDepot(EstablishDepotOrder)`.
  - Added `EstablishDepotOrder { submitter, area }`.
  - Updated `submitter()`, `corps()`, and movement classification.
- `supply.rs`
  - `trace_supply(&Scenario, &CorpsId) -> SupplyState`
  - `resolve_supply_phase(&mut Scenario, &AttritionTable) -> Vec<Event>`
  - `validate_depot_order(&Scenario, &EstablishDepotOrder) -> Result<(), String>`
- `lib.rs` now exports the supply surface.
- Existing economy/movement helper matches updated for the new order variant.

### Data and docs

- Added `docs/rules/supply.md` as the canonical supply reference.
- Added `data/tables/attrition.json` placeholder table.
- This phase report documents the gate closure.

## Test coverage

`crates/core/src/supply.rs::tests` now covers 36 hand-written cases, including:

- at-capital, one-hop, and multi-hop in-supply traces,
- enemy-owned-area and enemy-ZoC blocking,
- neutral-chain traversal,
- foraging vs placeholder-yield denial,
- deterministic repeated resolution,
- attrition application / skipping / clamping,
- deterministic BTreeMap processing order,
- depot validation across owned, friendly, neutral, enemy, and invalid areas.

## ADRs added

None.

## Adjudications added

None.

## Open questions

No new blocking questions were introduced for Phase 5. Depot *validation* is in
place, but depot *state tracking* is still absent from `Scenario`, so the live
trace currently resolves to capitals only unless and until depot persistence is
added in a later scoped phase.

## Known defects and caveats

- `Scenario` still has no explicit depot collection, so `trace_supply` follows
  the task instruction and traces to capitals only for now.
- Attrition row lookup currently prefers `default`, then `OUT_OF_SUPPLY`, per
  the task brief. Terrain/season-specific keyed lookup is deferred until the
  table data and the matching phase scope are ready.
- The legacy boolean `Corps.supplied` is updated as a convenience (`true` for
  `InSupply` and `Foraging`, `false` for `OutOfSupply`), but the richer source
  of truth for Phase 5 decisions is `SupplyState`.

## Next phase

Proceed only under an explicit next-phase brief. Phase 5 itself is complete:
core rules, docs, placeholder table, schemas, and test gate are all in place.
