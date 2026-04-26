# Phase 11 — AI

Date closed: 2026-04-25
Branch: `phase11-ai`
Gate: `docs/PROMPT.md` §15 / Phase 11

## Summary

Implemented a deterministic AI crate that operates on projected scenarios only, emits typed movement/economic/diplomatic orders, loads a default personality profile from data, and includes 20+ hand-written tests covering movement, economy, diplomacy, determinism, and helper behavior.

## Gate evidence

| Requirement | Status |
|---|---|
| Deterministic heuristics only | ✅ `str_hash`, ordered iteration, no floats, BTreeMap fixtures |
| Projection-only inputs | ✅ `AiContext` takes `&Scenario` projection only |
| Personality config | ✅ `AiPersonality` + `data/ai/personalities/default.json` |
| Movement heuristics | ✅ hold-if-threatened, otherwise move toward capital |
| Economic heuristics | ✅ build corps only when treasury/manpower/queue conditions allow |
| Diplomatic heuristics | ✅ aggressive personalities emit `DeclareWar` against `UNFRIENDLY` powers |
| 20+ tests | ✅ 25 tests in `crates/ai/src/lib.rs` |
| Clean build | ✅ fmt + clippy + full workspace test pass |

## What was built

### `crates/ai`

- Replaced the Phase-0 scaffold with a functioning AI module.
- Added:
  - `AiPersonality`
  - `AiContext`
  - `AiOrders`
  - deterministic helpers for adjacency, enemy detection, capital distance, and personality fallback loading
  - `generate_orders`
- Added 25 unit tests.

### Order vocabulary support

- Added `Order::DeclareWar(DeclareWarOrder)` so the AI can emit the diplomatic order required by the Phase 11 spec.
- Updated non-movement / non-economic match arms accordingly so the rest of the workspace remains exhaustive and compiling.

### Data and docs

- Added `data/ai/personalities/default.json`.
- Added `docs/rules/ai.md` documenting the four-layer model, deterministic seeding, projection-only behavior, and no-invented-values rule.

## Notes

- Corps build composition is derived from an existing visible corps owned by the acting power; this preserves the “no invented values” rule.
- Diplomatic AI intentionally emits at most one declaration per turn.

## Next phase

Future work can expand AI heuristics and personality packs, but Phase 11 itself is now implemented and covered by tests.
