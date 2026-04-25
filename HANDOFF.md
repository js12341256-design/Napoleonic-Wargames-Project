# Handoff briefing — Grand Campaign 1805

**For:** any coding agent or human picking up this repo.
**From:** the previous Claude Code session.
**Date:** 2026-04-25 (sandbox calendar; see Q9 for the real-world note).

Read this once, end-to-end, before touching any code.  Then read
`docs/PROMPT.md` end-to-end.  Both are short.

---

## 1. What this project is

A clean-room, original-IP digital recreation of a 1983 Napoleonic-era
grand-strategy board game (working title **Grand Campaign 1805**).
7 great powers, ~50 minors, monthly turns 1805–1815.  Multi-mode:
hotseat, LAN, online, PBEM, single-player vs AI.

The canonical brief is `docs/PROMPT.md`.  It is the single source of
truth.  Everything else in this repo is in service of it.

The codebase is a Rust workspace with a TS/React web shell to come.
See `README.md` for the layout.

## 2. State as of this handoff

| Phase | Status | Tests | Commit |
|-------|--------|-------|--------|
| 0 — Scaffolding              | ✅ green       | 0    | `c00d846` |
| 1 — Data model + loader      | ✅ green       | 39   | `6e3d612` |
| 2 — Map + movement           | ✅ green       | 70   | `0d7212d` |
| CI fixes (YAML + toolchain)  | ✅ pushed      | —    | `7863c4f`, `a764105` |
| 3 — Economy                  | 🟡 50% (rules doc, schema, orders, loader init done; resolver, tests, CLI not yet) | 70 | uncommitted at handoff time |
| 4–13                         | ⬜ not started | —    | — |
| 14–19 (UI / locales / polish)| ⬜ not started | —    | — |

**Latest green commit on `claude/implement-design-system-ciX1R`:** `a764105`.

Tests as of this handoff: 70 (17 schema + 46 core + 6 integration + 1
core-validate).  Workspace builds clean under `cargo build --workspace
--all-targets`, `cargo test --workspace`, `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`.

## 3. The eight §23 decisions (recorded in ADR 0001)

The master prompt §23 requires answers before Phase 0 starts.  The
user said "choose for me," so the answers were defaulted:

1. **Legal posture C** — clean-room original IP, no *Empires in Arms*
   names, art, or verbatim rules.
2. **License** — repo private, `LICENSE` deliberately absent (all
   rights reserved); decide before any public release.
3. **Reference hardware** — Ryzen 5 5600 / 16 GB / iGPU / 1080p.
4. **Designer cadence** — **unresolved** (questions.md Q1).  Phase 4
   onward cannot close its gates faithfully without a designer
   authoring `data/tables/*.json`.
5. **Determinism seed** — `0x5EEDED0000000001`.
6. **§1.5 non-goals** — unchanged.
7. **Continental System (§7.11)** — out of v1.0.
8. **Named events (§7.8)** — in v1.0.

## 4. Open blockers (questions.md)

These remain open and gate specific phases:

- **Q1 — Rules-table author.**  Blocks Phase 4 onward.  Without a
  designer, every numeric in `data/tables/*.json` stays
  `Maybe::Placeholder` and the 1805 scenario carries
  `unplayable_in_release: true`.  Per PROMPT.md §6.1 the agent does
  not invent these values.
- **Q2 — `docs/SPEC.md` contents.**  Defaulted to "PROMPT.md is the
  spec"; gaps surface as code is written.
- **Q3 — CI host.**  Defaulted to GitHub Actions.
- **Q4 — License decision.**  Blocks any public release.
- **Q5 — Tutorial scenario design.**  Blocks Phase 14.
- **Q6 — Definitive 1805 minor list.**  Blocks Phase 8.  Currently
  only Bavaria is included.
- **Q7 — Locale translators.**  Blocks Phase 16.
- **Q8 — Reference-hardware access.**  Blocks Phase 17.7 perf gates.
- **Q9 — Rust toolchain pin.**  `rust-toolchain.toml` currently
  tracks `stable` rather than a pinned minor version per §3.6.  The
  initial `1.94.1` pin failed because that version doesn't exist on
  real-world GitHub runners — the agent's sandbox simulates a 2026
  calendar.  Pick a real, current stable version and ADR-record it.

## 5. Hard rules the next agent MUST respect

These come straight from `docs/PROMPT.md` §0 and are non-negotiable:

1. **No invented numerical values.**  Combat odds, attrition rates,
   tax multipliers, peace-conference scoring, weather distributions,
   leader casualty rolls — none.  Missing values stay
   `Maybe::Placeholder` and the scenario flag is set.
2. **Determinism is sacred.**  No floats in the simulation core, no
   wall-clock, no hash-ordered iteration, fixed-point integers for
   anything that compares (morale is `i32 / 10000`).  See §2.
3. **Tests before implementation.**  Hand-written cases first, then
   the function.  20+ cases is the typical phase-gate floor.
4. **Headless before visual.**  No Bevy / web work until the
   simulation core runs AI-vs-AI to completion in CI.  Phases 14–15
   are post-Phase-11.
5. **Stop and ask** when stuck.  Add to `docs/questions.md` rather
   than guessing.
6. **One subsystem at a time** *was* the original rule, but in this
   session the user explicitly granted the agent permission to
   continue from phase to phase without human-review pauses.  All
   other rules still stand.  See conversation history.

## 6. The remaining plan

Per master prompt §16, in order, with a brief note on what each phase
needs:

- **Phase 3 — Economy.**  In progress.  Resolver, tests (20+ income
  cases), CLI subcommand `gc1805 economic-phase` still to do.  See
  `docs/rules/economy.md` for the spec.  Schema additions to
  `Scenario` (power_state, production_queue, replacement_queue,
  subsidy_queue, current_turn) and to `EconomyTable` are already
  drafted in WIP.  Order types
  (`SetTaxPolicy`/`BuildCorps`/`BuildFleet`/`Subsidize`) are drafted.
  Loader is updated to initialise `power_state` from
  `PowerSetup.starting_*`.
- **Phase 4 — Land combat.**  Hardest gate after Phase 1.  Needs
  `combat.json` from a designer.  Structurally: ratio buckets,
  formation matrix, terrain modifiers, retreat pathing, leader
  casualty, morale updates.  50+ hand-written cases at gate.
- **Phase 5 — Supply.**  Trace from capital and depots; foraging;
  attrition.  30+ topology cases.
- **Phase 6 — Diplomacy.**  Secret orders, conditional grammar,
  9-step resolution order, alliance cascade.  This is the most
  complex non-combat phase.
- **Phase 7 — Political.**  PPs, revolts, peace conferences,
  abdication.
- **Phase 8 — Minors.**  State machine (independent / allied / feudal
  / conquered / in-revolt), activation tables, Iberian guerilla.
  Blocked on Q6 for the full minor list.
- **Phase 9 — Naval.**  Fleets, naval combat, blockade, transport,
  weather.
- **Phase 10 — Full turn loop.**  Orchestrator, cross-platform
  determinism check.
- **Phase 11 — AI.**  Strategic / operational / tactical /
  diplomatic layers, deterministic, projection-only.
- **Phase 12 — Server.**  axum + tokio, WebSocket live-play,
  reconnect, secret-order projection.
- **Phase 13 — PBEM.**  Ed25519-signed envelopes.
- **Phase 14 — Desktop UI.**  Bevy.  First playable surface.
- **Phase 15 — Web UI.**  Rust→WASM core + TS/React shell under
  `/web`.
- **Phase 16 — Localization.**  7 locales (EN canonical).
- **Phase 17 — Modding.**  Load mods from `mods/`.
- **Phase 18 — Replay.**  Post-game viewer.
- **Phase 19 — Polish.**  Closed beta.

## 7. Repository conventions worth knowing

- **IDs are stable strings, prefixed by kind**: `AREA_PARIS`,
  `MINOR_BAVARIA`, `LEADER_NAPOLEON`, `CORPS_FRA_001`, etc.
- **`Scenario` is both initial state and live state.**  After load
  it gets mutated by `resolve_order` and (eventually) by the
  economic / combat / diplomatic resolvers.  This conflates initial
  and current; a future `WorldState` split is on the wishlist.
- **`Maybe<T>` lives in `gc1805_core_schema::tables`** and is the
  canonical "designer-authored or PLACEHOLDER" container.  Default
  is `Placeholder` so structures derive `Default` cleanly.
- **`Event` enum is event-sourced.**  Append-only, never mutate,
  never reorder.  See `crates/core-schema/src/events.rs`.
- **Determinism tiebreaker**: lexicographic on entity IDs at every
  sort/relax step.  `BTreeMap`/`BTreeSet`, never `HashMap`.
- **CLI binary is `gc1805`** under `crates/cli`.
- **Schemas are dumped via `cargo run -p xtask -- dump-schemas
  data/schemas`.**  Re-run after any schema-type change.

## 8. How to bring the next agent up to speed

The shortest path:

1. Clone the repo and check out `claude/implement-design-system-ciX1R`.
2. Read this file.
3. Read `docs/PROMPT.md` end-to-end.  No exceptions.
4. Read `docs/decisions.md` (ADR 0001).
5. Read `docs/questions.md` (Q1–Q9).
6. Read `docs/adjudications.md` (only one entry so far — interception
   scope at Phase 2).
7. Read every file under `docs/phase-reports/` in order.  These tell
   you what was actually built, what tests exist, what's known to be
   defective.
8. Skim `docs/rules/movement.md` and `docs/rules/economy.md` so you
   know the citation pattern §21.5 expects.
9. Run `cargo build --workspace && cargo test --workspace` to
   confirm the local environment is happy.
10. Read the current branch's git log: `git log --oneline main..HEAD`.

After that you should know enough to either:

- continue Phase 3 from `crates/core/src/economy.rs` (doesn't exist
  yet — create it; see economy.md for the algorithm), or
- if the user wants you to pick a different phase, start there with
  the same pattern: rules doc → schema → orders → resolver → 20+
  hand-written tests → CLI subcommand if relevant → phase report.

## 9. The agent contract

The previous session followed these self-imposed contracts on top of
PROMPT.md.  Honoring them keeps the codebase coherent:

- One commit per phase (or per logical chunk if a phase is huge).
  Commit messages start with `Phase N:` for phase work, `RULES:` if
  rules tables change (per PROMPT.md §6.4), `ci:` for CI changes.
- Phase reports go in `docs/phase-reports/phase-NN.md`.  CHANGELOG
  gets updated under `## [Unreleased]`.
- Tags `phase-NN-complete` are local-only on this remote (the test
  remote rejects tag pushes with HTTP 403).
- `docs/decisions.md` gains an ADR for any non-obvious architectural
  choice.  `docs/adjudications.md` gains an entry for any rules
  ambiguity that had to be resolved without designer input.

## 10. Things that are subtle / easy to break

- **`Scenario` has many `#[serde(default)]` fields now (current_turn,
  power_state, production_queue, …).**  Existing JSON is forwards-
  compatible, but new test fixtures using struct-literal syntax must
  fill all fields explicitly — Rust's struct literals don't honor
  serde defaults.  The pattern is `..Default::default()` or per-field
  defaults.
- **Movement `Order::corps()` returns `Option<&CorpsId>`.**  Economic
  orders return `None`.  Anywhere in the codebase that expected
  `&CorpsId` directly needs unwrapping.  `Order::is_movement()` is
  the convenience predicate.
- **Canonical-JSON serialization rejects floats and top-level nulls.**
  In-object nulls are dropped silently.  See
  `crates/core-schema/src/canonical.rs`.
- **`MapGraph::shortest_path_cost` skips placeholder edges.**  Don't
  expect `None` when a placeholder edge exists; only when no
  known-cost path reaches the destination.
- **`rust-toolchain.toml` tracks `stable`.**  If you pin a real
  version, update CI's `dtolnay/rust-toolchain` invocation to match
  AND ADR it.
- **CI YAML had a stray colon-in-string bug** that took multiple
  fixes to identify (`mapping values are not allowed here`).  When
  adding a `- run: echo "X: Y"` step in the future, single-quote the
  whole scalar or use a `|` block.

---

End of handoff briefing.  Good luck.
