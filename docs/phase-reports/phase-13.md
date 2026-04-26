# Phase 13 — PBEM

Date completed: 2026-04-25
Branch: `phase13-pbem`
Gate: `docs/PROMPT.md` §16.13

## Summary

Implemented the PBEM envelope/signing layer described by `docs/PROMPT.md` §10
and gated by §16.13.  Added Ed25519-signed `OrderEnvelope` support, host-side
collection state (`PbemHost`), signature verification, duplicate/turn/game
rejection paths, envelope-receipt checks, and 17 focused tests.  No actual
email delivery or other networking was added.

## Gate evidence

| §16.13 requirement | Status |
|---|---|
| Ed25519-signed PBEM order envelopes | ✅ `crates/netcode/src/lib.rs` |
| Envelope fields: player_id, game_id, turn, orders JSON, signature | ✅ `OrderEnvelope` |
| Deterministic message-to-sign format | ✅ `player_id:game_id:turn:orders_json` |
| Verification helper | ✅ `verify_envelope` |
| Host-mediated collection state | ✅ `PbemHost` |
| Reject wrong turn submissions | ✅ `collect_wrong_turn_rejected` |
| Reject duplicate player submissions | ✅ `collect_duplicate_player_rejected` |
| Reject invalid signatures | ✅ `collect_invalid_signature_rejected` |
| Detect when all expected players have submitted | ✅ `all_envelopes_received` + tests |
| No real networking / email sending | ✅ docs + implementation scope |
| 15+ tests | ✅ 17 tests |

## What was built

### `gc1805-netcode`

- Added dependencies:
  - `ed25519-dalek = "2"`
  - `serde_json = "1"`
  - `serde = { version = "1", features = ["derive"] }`
  - `gc1805-core-schema = { path = "../core-schema" }`
- Implemented `OrderEnvelope`
- Implemented `PbemHost`
- Implemented:
  - `sign_envelope`
  - `verify_envelope`
  - `collect_envelope`
  - `all_envelopes_received`
- Added 17 tests covering nominal flow, tampering, wrong key, wrong turn,
  wrong game, duplicate submissions, JSON serialization, deterministic signing,
  invalid signature length, and receipt edge cases.

### Documentation

- Added `docs/rules/pbem.md` describing PBEM envelope structure, verification
  flow, and host-mediated turn resolution with citations to `PROMPT.md` §10 and
  §16.13.

## Hard rules compliance

- ✅ No floats used
- ✅ No `HashMap` in PBEM logic
- ✅ No networking or email sending implemented
- ✅ Signature payload is deterministic string data only
- ✅ Test count exceeds requested minimum
