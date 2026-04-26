# Movement rules — canonical reference

PROMPT.md §21.5 requires every rules-implementing function to cite a
file under `docs/rules/`.  This is the canonical reference for Phase 2
(movement).  Every claim here is sourced from the master prompt; nothing
is invented.

## 1. Topology

The strategic map is a labelled, undirected graph:

- **Areas** are land vertices.  Every area carries an owner
  (`Owner::Power | Owner::Minor | Owner::Unowned`), a terrain (`Open`,
  `Forest`, `Mountain`, `Marsh`, `Urban`), and a fortification level.
- **Sea zones** are sea vertices.  Sea zones connect to other sea zones
  (`sea_adjacency`) and to land areas via `coast_links` for ports.
- **Adjacency** between areas is `Scenario.adjacency`, stored as
  `(from, to, cost)` and validated symmetric at load
  (`crates/core/src/validate.rs::check_adjacency`).

The scenario may store an edge cost as `Maybe::Placeholder` per
PROMPT.md §6.1.  Hop-count distances are always computable; cost-
weighted distances return `None` when any edge on the candidate path is
placeholder-valued.

## 2. Distance metrics

Two metrics are deterministic and both implemented:

- **Hop distance** — number of land edges separating two areas.  BFS
  from `from`; returns `None` if `to` is unreachable.  Iterates
  neighbours in a deterministic, sorted order so two equal-length paths
  always pick the lexicographically smallest predecessor (PROMPT.md
  §2.2).
- **Cost distance** — Dijkstra with the integer edge weights from
  `AreaAdjacency.cost`.  Returns `None` when any edge cost on the
  shortest path is placeholder, or when `to` is unreachable.

Sea distance and combined land/sea distance are out of scope until
Phase 9 (naval).

## 3. Movement orders

Phase 2 introduces two order kinds.  Both are validated by
`gc1805-core-validate`:

- `Order::Move { corps, to }` — the corps takes the shortest legal land
  path to `to`.  Validation requires `to` to be reachable in the
  current turn's movement budget.  In Phase 2 the budget is per-corps
  and uses hop count (`movement_hops_per_turn` in
  `Scenario.movement_rules`).
- `Order::ForcedMarch { corps, to }` — same but extends the per-turn
  budget by one hop, at the cost of `forced_march_attrition_q4` morale
  drop and an attrition SP loss roll on arrival.

Stacking (multiple corps in one area) is constrained by
`Scenario.movement_rules.max_corps_per_area`.  An order whose
destination would exceed the limit is rejected before resolution.

## 4. Interception (deferred)

`Order::Interception { corps, when, where }` is typed but only partially
validated in Phase 2.  Full resolution requires:

- supply trace from Phase 5,
- diplomatic-state lookup from Phase 6,
- impulse queue from Phase 10.

See `docs/adjudications.md` Adjudication 0001 for the chosen interim
behaviour.  In Phase 2, an `Interception` order validates structurally
(corps exists, target area exists, arming conditions are well-formed)
but the resolver returns `MovementResolution::Pending`.

## 5. Forbidden tricks

- Floats are forbidden in the simulation core.  Distances and costs are
  integers.  Morale is `i32 / 10000` (Q4 fixed-point).
- No iteration over hash-ordered containers.  `MapGraph` builds its
  adjacency lists from `BTreeMap`/sorted `Vec` to guarantee identical
  pathfinding tiebreaks across runs.
- Pathfinder tiebreaker is **lexicographic on AreaId**, applied at
  every relaxation step.

## 6. Test coverage

`testdata/rules_cases/movement/` holds the canonical rule cases.  Each
file is a YAML scenario describing a topology, an order, and the
expected outcome.  `crates/core/src/map.rs::tests` and
`crates/core-validate/src/movement_tests.rs` execute them.

Phase 2 ships with ≥ 20 cases covering adjacency lookups, hop and cost
pathfinding, stacking refusals, and forced-march framework calls.
