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

- Phase 8 — minors: integrated the full Q6 minor-country roster into the
  1805 scenario, added deterministic placeholder activation data in
  `data/tables/minors.json`, introduced `gc1805-core::minors`
  (`MinorStatus`, `activate_minor`, `validate_minor_control`), added
  `MinorActivated` / `MinorRevolt` events, authored `docs/rules/minors.md`,
  and closed Q6 in `docs/questions.md`.
- Phase 11 — deterministic AI heuristics in `gc1805-ai`, default personality
  config, projection-only order generation, `DeclareWar` order support, and
  25 hand-written tests.  See `docs/phase-reports/phase-11.md`.
- Phase 13 — PBEM: Ed25519-signed `OrderEnvelope` support,
  `PbemHost` collection state, signature verification, host-side turn/game/
  duplicate submission checks, PBEM rules documentation, and 17 tests.  See
  `docs/phase-reports/phase-13.md`.
- Phase 15 — Web UI: `gc1805-client-web` Rust/WASM bridge, `WasmGame` JSON interface, React/Vite shell in `web/`, `docs/rules/ui_web.md`, and 12 client-web tests. See `docs/phase-reports/phase-15.md`.
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
- Phase 5 — supply tracing and attrition resolution, `SupplyState`
  schema/events, `EstablishDepot` order validation, placeholder
  `attrition.json`, canonical `docs/rules/supply.md`, and 36 hand-
  written supply tests.  128 tests passing total.  See
  `docs/phase-reports/phase-05.md`.
- Phase 6 — diplomacy validator/resolver, diplomacy order vocabulary
  (`DeclareWar`, `ProposePeace`, `FormAlliance`, `BreakAlliance`),
  diplomacy event vocabulary, alliance cascade handling, and 30
  hand-written tests.  See `docs/phase-reports/phase-06.md`.
- Phase 7 — political phase: PP tracking in `power_state.prestige`,
  `apply_pp_delta` with `PpModifiersTable` lookup (all Placeholder),
  revolt triggers (`check_revolts`), abdication condition
  (`check_abdication`), `resolve_political_phase` entry point.
  `PrestigeAwarded`, `RevoltTriggered`, `PeaceConferenceOpened`,
  `AbdicationForced` event variants.  30 hand-written test cases,
  `docs/rules/political.md`.  All thresholds are structural placeholders.
  See `docs/phase-reports/phase-07.md`.
- Phase 9 — naval system skeleton: sea-zone graph, fleet movement,
  blockade detection, embark/disembark transport flow, naval order
  vocabulary, `NavalOutcome`, placeholder-authored naval combat/weather
  tables, `docs/rules/naval.md`, and 37 naval tests in
  `crates/core/src/naval.rs`.  Gate OPEN pending designer-authored
  naval combat and weather values.  See `docs/phase-reports/phase-09.md`.
- Phase 10 — full turn orchestration: `turn_loop.rs`, `AllTables`, `TurnInput`,
  `TurnOutput`, `run_turn`, new lifecycle events (`TurnStarted`,
  `PhaseCompleted`, `TurnCompleted`), 24 hand-written tests, and turn-loop
  rules documentation.  See `docs/phase-reports/phase-10.md`.
- Phase 12 — server scaffold: axum HTTP + WebSocket server, in-memory
  `GameSession` storage keyed by `game_id`, order submission routing,
  reconnect replay via `GET /games/{id}/events?since=N` and WebSocket
  `last_event_index`, player-specific `ProjectedScenario` responses,
  `docs/rules/server.md`, and 15+ server tests.  See
  `docs/phase-reports/phase-12.md`.
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
- Supply resolution now emits `SupplyTraced` and `AttritionApplied`
  events, and the core schema exports `SupplyState` with canonical JSON
  tags `IN_SUPPLY`, `FORAGING`, and `OUT_OF_SUPPLY`.
