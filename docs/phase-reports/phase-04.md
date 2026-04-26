# Phase 4 — Land Combat

Date completed: 2026-04-25
Branch: `integrate/q1-combat-tables-q9-toolchain`
Gate: `docs/PROMPT.md` §16.4

## Summary

Land combat resolver covering: force ratio → bucket selection (integer
arithmetic, no floats), column shifts (formation + terrain), die-roll
result lookup, SP loss distribution, morale delta application, outcome
determination (rout / retreat / repulsed / mutual withdrawal), and
retreat resolution via lexicographic ZoC-filtered candidates.  52
hand-written test cases.

**Gate status: CLOSED — Q1 answered on 2026-04-25.**  
Designer-authored `combat.json` values are now integrated in the repo and the resolver/schema consume the real table shape directly. Integrated by commit `eab2577`.

Workspace clean under fmt, build, test, and clippy on Rust 1.95.0.

## Gate evidence

| §16.4 requirement | Status |
|---|---|
| Force ratio → bucket (6 buckets, integer arithmetic) | ✅ Tests 34–39 in `combat.rs::tests` |
| Formation column shifts looked up from table | ✅ Tests 40–43 |
| Terrain column shifts looked up from table | ✅ Test 40 |
| Die-index clamping (low and high) | ✅ Tests 42–43 |
| Missing combat-table bucket → OrderRejected with code COMBAT_TABLE_PLACEHOLDER | ✅ Test 26 |
| SP loss applied to attacker and defender | ✅ Tests 28–29 |
| Morale delta applied to both sides | ✅ Test 30 |
| DefenderRetreats outcome | ✅ Test 31 |
| DefenderRouted outcome + CorpsRouted event | ✅ Tests 32, 48 |
| AttackerRepulsed outcome | ✅ Test 33 |
| MutualWithdrawal outcome (all fields correct) | ✅ Test 51 |
| CorpsRetreated event (retreat to adjacent non-ZoC area) | ✅ Test 49 |
| Corps with 0 SP stays in map (removal is Phase 10) | ✅ Test 52 |
| Determinism: same seed → same events | ✅ Test 44 |
| No-defender → OrderRejected with code NO_DEFENDER | ✅ Test 50 |
| ZoC: empty without corps | ✅ Test 16 |
| ZoC: adjacency union minus own areas | ✅ Tests 17–25 |
| Validate: all 8 preconditions tested | ✅ Tests 1–15 |
| 50+ hand-written cases | ✅ 52 cases in `combat.rs::tests` |
| combat.json: designer-authored values integrated | ✅ `data/tables/combat.json` |
| attrition.json, morale.json, leader_casualty.json created | ✅ `data/tables/` |

## Gate closure

**Q1 resolved.** The designer-authored combat table is now installed at
`data/tables/combat.json`, and the combat schema/resolver were updated to
consume the actual authored format (`ratio_buckets` objects, formation metadata,
and bucket-id keyed `results_table`).

## What was built

### `gc1805-core-schema`

- `combat_types.rs` — New module:
  - `BattleOutcome` enum: `AttackerRepulsed`, `DefenderRetreats`,
    `DefenderRouted`, `MutualWithdrawal`
  - `LeaderCasualtyKind` enum: `Unharmed`, `Wounded`, `Killed`
- `events.rs` — Four new `Event` variants:
  - `BattleResolved { area, attacker, defender, attacker_sp_before,
    defender_sp_before, attacker_sp_loss, defender_sp_loss,
    attacker_morale_q4_delta, defender_morale_q4_delta, outcome }`
  - `CorpsRetreated { corps, from, to }`
  - `CorpsRouted { corps, area }`
  - `LeaderCasualty { leader, casualty_kind }`
  Note: field renamed `casualty_kind` (not `kind`) to avoid collision
  with serde's `#[serde(tag = "kind")]` on the Event enum.
- `lib.rs` — Re-exports `BattleOutcome`, `LeaderCasualtyKind`.

### `gc1805-core`

- `orders.rs` — New order types:
  - `AttackOrder { submitter, attacking_corps: Vec<CorpsId>, target_area, formation }`
  - `BombardOrder { submitter, corps, target_area }`
  - `Order::Attack(AttackOrder)` and `Order::Bombard(BombardOrder)` variants
  - `Order::submitter()`, `Order::corps()`, `Order::is_movement()` updated
- `combat.rs` — New module with:
  - `zones_of_control(&Scenario, &PowerId) -> BTreeSet<AreaId>` — pure query
  - `validate_attack(&Scenario, &AttackOrder) -> Result<(), String>` — 8 checks
  - `resolve_battle(&mut Scenario, &CombatTable, &MoraleTable, u64, &AttackOrder) -> Vec<Event>`
    — full battle resolution: ratio bucket, column shifts, die roll, SP loss
    distribution, morale deltas, outcome determination, retreat resolution
  - 52 unit tests (validation 1–15, ZoC 16–25, resolution 26–52)
- `lib.rs` — `pub mod combat;` added
- `movement.rs` — Attack/Bombard arms added to the non-movement order match

### `data/tables/`

- `combat.json` — Designer-authored combat table integrated from Q1 answer
- `attrition.json` — Empty rows map (placeholder until designer authors values)
- `morale.json` — All three thresholds `{"_placeholder": true}`
- `leader_casualty.json` — Empty intensity map

### `docs/rules/`

- `combat.md` — Full rules reference for land combat: ratio buckets,
  column shifts, die lookup, SP loss, morale, outcome determination,
  retreat resolution, leader casualty, ZoC definition

## Hard rules compliance

- ✅ No floats anywhere (ratio bucket uses integer comparisons only)
- ✅ No invented numerical values in code; production combat values are designer-authored in `data/tables/combat.json`
- ✅ 52 hand-written test cases
- ✅ BTreeMap/BTreeSet only in simulation logic (no HashMap)
- ✅ Phase 5 not started
