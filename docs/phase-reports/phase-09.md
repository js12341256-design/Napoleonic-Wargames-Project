# Phase 09 Report — Naval

## Scope

Implemented the Phase 9 naval skeleton described in `docs/PROMPT.md` §16.9.

## Delivered

- `docs/rules/naval.md` covering sea graph movement, port access via `CoastLink`, blockade, transport, placeholder naval combat, and placeholder weather.
- New placeholder-authored rules data:
  - `data/tables/naval_combat.json`
  - `data/tables/weather.json`
- Schema additions:
  - `NavalOutcome` in `crates/core-schema/src/naval_types.rs`
  - naval `Event` variants for fleet movement, blockade, battles, embark, and disembark
  - `Fleet.embarked_corps` persistence field
- Core order additions:
  - `MoveFleet`
  - `NavalAttack`
  - `Embark`
  - `Disembark`
- New core naval module:
  - `SeaGraph`
  - fleet-move validation and resolution
  - placeholder-aware naval battle resolution
  - embark/disembark validation and resolution
- Test coverage: 37 naval tests in `crates/core/src/naval.rs`.

## Determinism notes

- No floating-point arithmetic used.
- Ordered containers only (`BTreeMap` / `BTreeSet`) for naval simulation logic.
- Naval battle die selection is deterministic from `rng_seed % die_faces`.
- Placeholder naval table entries reject resolution instead of inventing values.

## Validation

Ran successfully:

```sh
source ~/.cargo/env && cargo fmt --all
source ~/.cargo/env && cargo clippy --workspace --all-targets -- -D warnings
source ~/.cargo/env && cargo test --workspace
```

## Gate status

Phase 9 code is in place, but naval combat and weather tables remain placeholder-authored until designers provide final data.
