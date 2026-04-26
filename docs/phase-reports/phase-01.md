# Phase 1 — Data model and scenario loader

Date closed: 2026-04-25
Branch: `claude/implement-design-system-ciX1R`
Gate: `docs/PROMPT.md` §16.2

## Summary

Typed scenario schema in place, canonical-JSON pipeline working, fog-of-
war projection function with 11 hand-written cases, the 1805 scenario
present (with `PLACEHOLDER` values, as the gate explicitly permits), and
JSON Schemas dumped for every persisted root.

Tests: 39 passing (17 schema, 16 core, 6 integration).  Workspace clean
under `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy
-D warnings`.

## Gate evidence

| §16.2 requirement                                                          | Status                                                       |
|----------------------------------------------------------------------------|--------------------------------------------------------------|
| All §5 types implemented with serde derive and canonical serialization     | ✅ `gc1805-core-schema::{ids,scenario,tables,canonical}`     |
| `data/scenarios/1805_standard/` complete (placeholders OK for unit values) | ✅ `scenario.json` loads cleanly; `unplayable_in_release: true` |
| Round-trip test: load → re-serialize → reload → hashes match               | ✅ `tests/load_1805.rs::round_trip_canonical_hash_stable`    |
| JSON schemas via `schemars` committed                                      | ✅ `data/schemas/*.schema.json` (10 files)                   |
| Projection function with ≥10 hand-written fog-of-war cases                 | ✅ 11 cases in `crates/core/src/projection.rs::tests`        |

Cross-platform verification still owned by CI.  Phase 1 builds locally
on Linux x86_64 in 0.36 s incremental.

## What was built

### `gc1805-core-schema`

- `ids.rs` — seven stable-string newtypes (`PowerId`, `MinorId`,
  `LeaderId`, `AreaId`, `SeaZoneId`, `CorpsId`, `FleetId`) generated
  via a `id_newtype!` macro.  Each has `PREFIX`, `validate_id`, and
  full `Ord`/`Serialize`/`Deserialize`/`JsonSchema`.
- `canonical.rs` — `to_canonical_string`, `to_canonical_bytes`,
  `canonical_hash` (BLAKE3 hex), `CanonicalJsonError`.  Implements
  PROMPT.md §5.2 byte-for-byte: keys sorted, nulls omitted,
  integer-only numbers, control-character escapes, no float fallback.
- `scenario.rs` — `Scenario` root plus `PowerSetup`, `MinorSetup`,
  `Leader`, `Area`, `SeaZone`, `Corps`, `Fleet`, `Owner`,
  `DiplomaticState`, `DiplomaticPairKey` (lexicographic-pair keying),
  `Terrain`, `MinorRelationship`, `Features`, `GameDate`,
  `AreaAdjacency`, `CoastLink`, `SeaAdjacency`.
- `tables.rs` — `Maybe<T>` (designer-authored value or
  `{"_placeholder": true}` marker) plus `CombatTable`,
  `AttritionTable`, `WeatherTable`, `MinorActivationTable`,
  `PpModifiersTable`, `LeaderCasualtyTable`, `MoraleTable`,
  `NavalCombatTable`, `EconomyTable`.

`Maybe` is also used in `Area.{money_yield, manpower_yield}` and
`AreaAdjacency.cost` because those are designer-authored numerics that
the gate permits as placeholders.

### `gc1805-core`

- `loader.rs` — `load_scenario_str`, `LoadError`, `LoadReport`.  The
  loader scans for placeholders before typed deserialization and
  surfaces the dotted JSON paths.  Schema-version mismatches raise
  hard errors.
- `validate.rs` — `validate_scenario` returns a `Vec<IntegrityIssue>`
  covering ID shape, capital and ruler references, minor patrons,
  unit locations, and adjacency symmetry.  Designed to be incremental
  — Phase 2 (movement) and Phase 6 (diplomacy) will extend the issue
  set without changing the public API.
- `projection.rs` — `project(full, viewer) → ProjectedScenario`.
  Static fog-of-war: areas/powers/leaders/sea-zones always visible;
  diplomacy filtered to viewer-touching pairs plus all `WAR` pairs;
  corps and fleets visible if owned by viewer or sitting in a
  viewer-owned area.  11 unit tests + 1 integration test exercise
  the rules.

### `data/scenarios/1805_standard/scenario.json`

A hand-authored 1805 starting state with:

- 7 majors (FRA, GBR, AUS, PRU, RUS, SPA, OTT) with capitals, rulers,
  mobilization areas, max-corps, max-depots.
- 17 leaders with strategic / tactical / initiative ratings authored
  from generally-known historical reputation (these are *not* designer-
  blessed ratings; they exist to populate the type system, and the
  scenario carries `unplayable_in_release: true`).
- 13 areas — France 5, Britain 1, Austria 2, Prussia 1, Russia 2,
  Spain 1, Ottoman 1, Bavaria-minor 1.  Every area's `money_yield` and
  `manpower_yield` is a `PLACEHOLDER` until a designer authors them.
- 4 sea zones (English Channel, North Sea, Baltic, Western Med).
- 6 corps (one per major except SPA/OTT, plus a second French corps).
- 2 fleets (GBR Channel Fleet, French Brest squadron).
- 1 minor (Bavaria, allied-free to FRA) — actual minor list is Q6.
- Diplomatic state: France ↔ Britain at WAR; rest of Europe in
  scenario-realistic neutrality / friendliness.
- Adjacency: 10 reciprocal land pairs covering the arteries Paris–
  Vienna–Berlin–St Pete–Moscow.  Costs are `PLACEHOLDER`.

The placeholder paths surface in `LoadReport.placeholder_paths` and
the integration test in `crates/core/tests/load_1805.rs` asserts that
the file loads successfully *and* surfaces placeholders.

### `tools/xtask`

- `dump-schemas <out_dir>` subcommand emits 10 JSON Schema files (one
  per persisted root) using `schemars`.  Run as
  `cargo run -p xtask -- dump-schemas data/schemas`.

### `data/schemas/`

Generated from the live Rust types — kept in source so PR reviewers
can see schema drift without running cargo.  Regenerate any time the
type definitions change.

### CI

The CI workflow continues to run as authored in Phase 0 — fmt, clippy,
build/test matrix.  No CI changes were required; the additional
deps and tests slot in transparently.

## ADRs added

None.  All decisions in Phase 1 followed straight from PROMPT.md §5–§6
and ADR 0001.  Specifically:

- `Maybe<T>` representation (`{"_placeholder": true}` literal) follows
  PROMPT.md §6.1 verbatim.
- `DiplomaticPairKey` ordering (lexicographic-min first) follows §2.2.
- `Owner` as a tagged enum follows the spec's expectation that ownership
  is one of `Power | Minor | Unowned`.

## Adjudications added

None.  No genuinely ambiguous rules were touched in Phase 1 — only the
schema shape and round-trip mechanics.

## Open questions still blocking

`docs/questions.md`:

- **Q1 (rules tables author)** — placeholders persist; scenario stays
  `unplayable_in_release: true`.  Phase 4 cannot start until this
  closes.
- **Q2 (SPEC.md contents)** — defaulted to "PROMPT.md is the spec";
  every gap will surface as code is written.
- **Q6 (minor list)** — only Bavaria is included.  Phase 8 cannot
  close until the full list arrives.

The other questions (Q3 CI host, Q4 license, Q5 tutorial, Q7
translators, Q8 hardware) do not block Phase 1.

## Known defects and caveats

- The leader ratings (`LEADER_NAPOLEON.strategic = 5`, etc.) are
  illustrative.  They should be replaced when the designer arrives, and
  the determinism golden regenerated.  Current values are flagged
  implicitly by `unplayable_in_release: true`.
- The `Corps`-in-scenario `morale_q4` field uses values like `9500`
  meaning "0.95" in Q4 fixed-point, but the convention is documented
  only in code.  Move this to `docs/rules/economy.md` (or a new
  `units.md`) when Phase 3 begins.
- `validate_scenario` does not yet check that `coast_links` reference
  `port: true` areas, nor that `fleets.at_port` always lands in a
  port.  Phase 9 (naval) will tighten this.
- The integration test reads `data/scenarios/1805_standard/scenario.json`
  via `CARGO_MANIFEST_DIR/../../`; this is brittle to layout changes.
  When `xtask` grows a `test-data-path` helper, switch to that.

## Next phase

Phase 2 — Map and movement (`docs/PROMPT.md` §16.3).  This is the
first phase that can run cleanly with placeholders (movement costs are
already typed as `Maybe<i32>`); it can begin without Q1 closing.
