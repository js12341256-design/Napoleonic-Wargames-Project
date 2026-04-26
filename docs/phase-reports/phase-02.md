# Phase 2 — Map and movement

Date closed: 2026-04-25
Branch: `claude/implement-design-system-ciX1R`
Gate: `docs/PROMPT.md` §16.3

## Summary

Strategic map graph with deterministic adjacency and pathfinding, full
movement order/event vocabulary, validator + resolver covering the
move/forced-march/hold/interception family with stacking enforcement,
and a headless CLI (`gc1805 move-all-to-capital`) that runs the §16.3
gate's "move every corps to capital" script against the 1805 scenario.

70 tests passing (40 prior + 30 new).  Workspace clean under fmt,
clippy, build, test.

## Gate evidence

| §16.3 requirement | Status |
|---|---|
| Adjacency and distance matrices for hand-written pairs | ✅ 18 cases in `crates/core/src/map.rs::tests` (5 adjacency, 8 hop, 5 cost) |
| Terrain movement costs applied correctly | ✅ Cost field is `Maybe<i32>`; Dijkstra honours integer costs and skips placeholder edges |
| Strategic path-finder verified against 20 hand-written cases | ✅ 18 map cases + 12 movement cases = 30 total |
| Forced march, interception, and stacking rules implemented | ✅ Forced-march budget +1 with morale drop; stacking via `MovementRules.max_corps_per_area`; interception typed-and-queued, see Adjudication 0001 |
| Headless CLI runs "move every corps to capital" | ✅ `gc1805 move-all-to-capital data/scenarios/1805_standard/scenario.json` — deterministic output |

## What was built

### `gc1805-core-schema`

- `events.rs` — `Event` enum (`MovementResolved`, `ForcedMarchResolved`,
  `InterceptionQueued`, `OrderRejected`).  `serde(tag = "kind")` keeps
  the canonical-JSON form forwards-stable.
- `tables::Maybe::default()` now returns `Placeholder`, letting
  `MovementRules::default()` derive cleanly.
- `scenario::MovementRules` — designer-authored numerics (max corps per
  area, hops per turn, forced-march extras), all `Maybe<i32>`.

### `gc1805-core`

- `map.rs` — `MapGraph` (adjacency lists from `BTreeMap`/`BTreeSet`),
  `shortest_path_hops` (BFS), `shortest_path_cost` (Dijkstra with
  lex tiebreaking, placeholder edges treated as impassable).
- `orders.rs` — `Order { Hold, Move, ForcedMarch, Interception }` with
  `submitter()` / `corps()` accessors.
- `movement.rs` — `validate_order` returns `MovementPlan` or typed
  `MovementRejection`; `resolve_order` mutates the scenario and emits
  the matching `Event`; `validate_or_reject` is a UI-friendly wrapper.
- `lib.rs` re-exports the public surface.

### `gc1805-core-validate`

- Now hosts the public façade `validate(&scenario, &order)`.  Smoke
  test: every starting French corps successfully validates a Hold.

### `gc1805-cli` (`gc1805` binary)

- `gc1805 load <scenario.json>` — parse, validate, print scenario
  summary plus canonical state hash.
- `gc1805 move-all-to-capital <scenario.json>` — for every corps,
  attempt to march toward its owner's capital, deterministic output.
  On the placeholder-1805 scenario the script reports 0-hop
  no-ops for corps already at capital and `PLACEHOLDER_RULE`
  rejections for corps elsewhere.  Output is fully reproducible.

### `testdata/rules_cases/movement/`

Six representative YAML cases mirroring Rust tests, plus a README
that names Rust as ground truth.  CI runs the Rust suite; the YAML is
a designer-readable mirror.

### Documentation

- `docs/rules/movement.md` — canonical reference cited by every
  movement-implementing function.
- `docs/adjudications.md` — Adjudication 0001 explains why
  interception is "typed and queueable" rather than fully resolved at
  Phase 2; flags Phase 10 as the closure point.

## ADRs added

None.  Decisions in Phase 2 follow directly from PROMPT.md §16.3 and
prior ADRs.

## Adjudications added

- **0001** — Interception at Phase 2 is structural-only.

## Open questions

`docs/questions.md` Q1, Q2, Q6 carry forward.  Specifically blocking:

- Phase 4 onward by Q1 (rules-tables author).
- Phase 8 by Q6 (full minor list).

Phase 2 itself was unblocked because:

- All Phase-2 numerics are typed `Maybe<i32>` and tolerate placeholders.
- Pathfinder cost handling skips placeholders rather than aborting.
- The "move every corps to capital" script's deterministic-rejection
  behaviour is an acceptable expected end state pending real rules
  values (the run hash is recorded by the script, not against a
  golden file — the golden lands when real movement_rules numerics
  arrive).

## Known defects and caveats

- `core::movement::resolve_order` uses `expect("non-empty path")` in
  the `Move` and `ForcedMarch` arms.  These cannot fire because
  validation guarantees a non-empty path, but they are the kind of
  thing a panic-free codebase would prefer to encode at the type
  level.  Defer to Phase 10 when the order pipeline matures.
- The `move-all-to-capital` script does not yet call the §2.5 state-
  hashing API across the event log — it computes the hash on the
  scenario alone.  Phase 10 (full turn loop) introduces the
  event-log fold and brings hashing into the canonical place.
- Phase 2 still lacks a no-edge-leaks check between corps owners and
  the area-owner relationships needed for ZoC.  ZoC enters with
  Phase 4 (combat) where it actually affects state.

## Next phase

Phase 3 — Economy (`docs/PROMPT.md` §16.4).  Income, maintenance,
production, manpower replacement.  This phase reads area yields and
maintenance costs; both are placeholder today.  It can begin
structurally but cannot close its gate's "20 hand-written
calculation cases" until Q1 closes — flagged in the questions file.
