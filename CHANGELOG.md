# Changelog

All notable changes to this project are recorded here.  This file follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and semantic
versioning as applied to the game rules (see `docs/PROMPT.md` §11.2).

Sections that may appear per release:

- **Added** — new features or scenarios.
- **Changed** — non-rules behavior changes.
- **Rules** — changes to `data/tables/*.json` or rules code that alter
  deterministic outcomes.  Each such release regenerates the determinism
  golden (see `docs/PROMPT.md` §2.6).
- **Fixed** — bug fixes.
- **Removed** — removed features.

---

## [Unreleased]

### Added

- Phase 0 scaffolding: workspace, 13 stub crates, directory layout per
  `docs/PROMPT.md` §4, CI workflow, toolchain pins, baseline documentation.
- `reference/prototype/` archived from the *Dusk of the Old World* design
  bundle as a visual-style reference only (see its `README.md`).
- ADR 0001 recording §23 answers and the prototype-vs-master-prompt
  contradiction resolution.
- Phase 1 — data model, canonical-JSON pipeline, scenario loader,
  fog-of-war projection, integrity validator, 1805 scenario placeholder,
  and `xtask dump-schemas`.  39 tests passing.  See
  `docs/phase-reports/phase-01.md`.
- Phase 2 — strategic-map graph, BFS hop pathfinder, Dijkstra cost
  pathfinder (placeholder-edge tolerant), `Order`/`Event` types,
  movement validator + resolver, `gc1805 move-all-to-capital` CLI
  subcommand.  70 tests passing (30 movement-related, 40 prior).  See
  `docs/phase-reports/phase-02.md`.
- Adjudication 0001 (`docs/adjudications.md`) — interception scope at
  Phase 2 reduced to "typed and queueable" pending impulse model.
- Phase 3 — economic phase resolver (income, maintenance, replacement
  queue, production queue, subsidies), economy order vocabulary
  (`SetTaxPolicy`, `BuildCorps`, `BuildFleet`, `Subsidize`), 22
  hand-written test cases, `gc1805 economic-phase` CLI subcommand.
  92 tests passing.  See `docs/phase-reports/phase-03.md`.
- Phase 4 — land combat: `BattleOutcome`/`LeaderCasualtyKind` types,
  `BattleResolved`/`CorpsRetreated`/`CorpsRouted`/`LeaderCasualty` event
  variants, `AttackOrder`/`BombardOrder` order types, `zones_of_control`,
  `validate_attack`, `resolve_battle` resolver skeleton (placeholder-tolerant),
  52 hand-written test cases, combat/attrition/morale/leader_casualty data
  table stubs, `docs/rules/combat.md`.  Gate OPEN pending Q1 (designer must
  provide real combat.json values).  144 tests passing.
  See `docs/phase-reports/phase-04.md`.
- Phase 14 — desktop UI Bevy skeleton: desktop client dependencies,
  main-menu and game-board placeholder screens, app-state helper module,
  `docs/rules/ui_desktop.md`, and `docs/phase-reports/phase-14.md`.

### Rules

- Schema version 1 introduced (`Scenario.schema_version = 1`).  No
  rules tables are filled yet; the 1805 scenario remains
  `unplayable_in_release: true` per PROMPT.md §6.1.
- `Scenario.movement_rules` added with four placeholder-friendly
  numerics: `max_corps_per_area`, `movement_hops_per_turn`,
  `forced_march_extra_hops`, `forced_march_morale_loss_q4`.
