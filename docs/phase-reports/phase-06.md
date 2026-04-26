# Phase 6 — Diplomacy

Date closed: 2026-04-25
Branch: `phase6-diplomacy`
Gate: `docs/PROMPT.md` §16.6

## Summary

Implemented the Phase 6 diplomacy slice for Grand Campaign 1805:
diplomacy rules documentation, diplomacy event/order vocabulary,
validation, deterministic phase resolution, alliance cascade handling,
and 30 hand-written tests in `crates/core/src/diplomacy.rs`.

Steps 1–4 of the nine-step diplomacy sequence are live. Steps 5–9 are
intentionally reserved no-ops pending later phases.

## Gate evidence

| §16.6 requirement | Status |
|---|---|
| Canonical diplomacy rules doc exists | ✅ `docs/rules/diplomacy.md` |
| Diplomacy event variants added | ✅ `WarDeclared`, `PeaceProposed`, `PeaceAccepted`, `AllianceFormed`, `AllianceBroken`, `PrestigeChanged`, `AllianceCascade` |
| Diplomacy order variants added | ✅ `DeclareWar`, `ProposePeace`, `FormAlliance`, `BreakAlliance` |
| Validator implemented | ✅ `validate_diplomatic_order` |
| Resolver implemented in deterministic order | ✅ `resolve_diplomatic_phase` |
| Canonical pair helpers implemented | ✅ `get_diplomatic_state`, `set_diplomatic_state` |
| Alliance cascade implemented | ✅ direct + chained alliance pulls covered by tests |
| 30+ hand-written tests | ✅ 30 tests in `diplomacy.rs::tests` |
| Placeholder-safe PP handling | ✅ only authored `Maybe::Value` deltas apply |
| Schema dump regenerated | ✅ `cargo run -p xtask -- dump-schemas data/schemas` |

## What was built

### `gc1805-core-schema`

- `events.rs`
  - Added diplomacy-facing `Event` variants for war, alliance, peace,
    prestige changes, and cascade entry.

### `gc1805-core`

- `orders.rs`
  - Added diplomacy order variants and structs:
    `DeclareWarOrder`, `ProposePeaceOrder`, `FormAllianceOrder`,
    `BreakAllianceOrder`.
  - Updated `submitter()` and `corps()` handling.
- `diplomacy.rs`
  - Added `get_diplomatic_state` and `set_diplomatic_state` using the
    canonical symmetric `DiplomaticPairKey`.
  - Added `validate_diplomatic_order`.
  - Added `resolve_diplomatic_phase` with fixed ordering:
    declare war → break alliance → form alliance → propose peace.
  - Added deterministic alliance cascade across alliance chains.
  - Added prestige / PP application for `declare_war` from
    `PpModifiersTable.events["declare_war"]`, skipping placeholders.
  - Added 30 unit tests covering validation, resolution ordering,
    placeholder/value PP behavior, canonical keys, empty resolution,
    same-turn break/form sequencing, and chained cascades.
- `lib.rs`
  - Exported the diplomacy module surface.
- `movement.rs`, `economy.rs`
  - Extended `Order` matching so the new diplomacy variants compile
    cleanly and are rejected in non-diplomatic pipelines.

### Documentation

- `docs/rules/diplomacy.md`
  - Added canonical diplomacy reference with §16.6 citations.
- `docs/phase-reports/phase-06.md`
  - Added this report.
- `CHANGELOG.md`
  - Updated Unreleased with Phase 6 entry.

## Adjudications added

None.

## Open questions / caveats

- The repository copy of `docs/PROMPT.md` is truncated before the full
  text of §16.6. This phase followed the explicit task brief as the
  operative statement of the §16.6 gate and cited §16.6 in the rules doc.
- `PeaceAccepted` is schema-ready but not resolved yet; peace acceptance
  remains intentionally deferred.
- Diplomatic PP hooks beyond `declare_war` remain table-driven but not yet
  executed because no additional semantics were specified.
- Secret diplomatic orders are represented as pending submitted orders and
  become public when the phase emits reveal events. No separate persisted
  proposal queue was added in this phase.

## Verification

Executed successfully:

```sh
source ~/.cargo/env && cargo fmt --all
source ~/.cargo/env && cargo clippy --workspace --all-targets -- -D warnings
source ~/.cargo/env && cargo test --workspace
source ~/.cargo/env && cargo run -p xtask -- dump-schemas data/schemas
```

## Next phase

Phase 7, which can build on the new diplomacy events and proposal flow to
add delayed peace acceptance and any remaining diplomatic side effects.
