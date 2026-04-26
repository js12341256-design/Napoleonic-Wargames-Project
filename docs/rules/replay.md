# Replay rules — canonical reference

Sourced from `docs/PROMPT.md` §16.18 and the event-log rule in §2.3.

## 1. Replay model

A replay is a deterministic fold of ordered events over an initial
scenario snapshot:

- input: `initial_scenario + Vec<Event>`
- output: the scenario state after replaying zero or more completed turns

Replay is read-only reconstruction. It does **not** call the full live
resolvers, because those resolvers may depend on broader mutable runtime
context. Instead, replay applies only the state changes already encoded
in the event log.

## 2. Replay file format

Replay files are JSON objects with these fields:

- `schema_version: u32`
- `game_id: String`
- `initial_scenario: Scenario`
- `events: Vec<Event>`
- `turn_hashes: Vec<TurnHash>`

`TurnHash` stores:

- `turn: u32`
- `hash: String` — 64-character hex BLAKE3 hash of the canonical scenario
  state for that seek target

The replay module serializes with `serde_json::to_string_pretty`. The
integrity hash itself is always derived from `canonical_hash`, not from
pretty-printed JSON formatting.

## 3. Seek-to-turn

`seek_to_turn(replay, n)` replays the event log from the start until the
requested seek target is reached.

- `n = 0` returns the untouched `initial_scenario`
- `n > 0` requires a matching entry in `turn_hashes`
- replay walks the event list in order and stops when it reaches the
  `TurnCompleted` marker that advances the scenario to the requested turn

This is strictly "replay N turns from start," not random mutation of an
already-live scenario.

## 4. Read-only event applier in Phase 18

Phase 18 replay reconstructs only the state transitions already encoded
in these events:

- `IncomePaid { power, net, .. }`
  - `scenario.power_state[power].treasury += net`
- `MaintenancePaid { power, corps_cost, fleet_cost }`
  - `scenario.power_state[power].treasury -= corps_cost + fleet_cost`
- `TreasuryInDeficit { power, .. }`
  - `scenario.power_state[power].treasury = 0`
- `MovementResolved { corps, to, .. }`
  - `scenario.corps[corps].area = to`
- `TurnCompleted { turn }`
  - `scenario.current_turn = turn + 1`

All other event variants are currently treated as replay no-ops in this
phase. That is intentional and should be extended in later phases as more
subsystems require replay-visible reconstruction.

## 5. Integrity check

Replay integrity is verified by re-deriving the scenario at each stored
seek target and hashing that reconstructed scenario with `canonical_hash`.

For every `TurnHash` entry:

1. `seek_to_turn(replay, turn_hash.turn)`
2. compute `canonical_hash(&scenario)`
3. compare with `turn_hash.hash`
4. fail on the first mismatch

This keeps replay verification deterministic and independent of JSON key
ordering or whitespace.

## 6. Constraints

- No floats
- No hash-ordered simulation logic
- Event order is authoritative
- Missing `TurnCompleted` markers are replay errors for hashed turns
- Missing referenced powers or corps are replay errors, not silent fixes
