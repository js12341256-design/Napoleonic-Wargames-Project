# Changelog

All notable changes to this project are recorded here. This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and semantic versioning as applied to the game rules (see `docs/PROMPT.md` §11.2).

Sections that may appear per release:

- **Added** — new features or scenarios.
- **Changed** — non-rules behavior changes.
- **Rules** — changes to `data/tables/*.json` or rules code that alter deterministic outcomes.
- **Fixed** — bug fixes.
- **Removed** — removed features.

---

## [Unreleased]

### Added

- **2026-04-24 — Phase 0 scaffolding:** workspace, 13 crates, directory layout, CI workflow, baseline docs, prototype archive, and repository bootstrap.
- **2026-04-25 — Phase 1 data model + loader:** canonical JSON, stable IDs, scenario loader, fog-of-war projection, integrity validator, 1805 placeholder scenario, JSON schema dumping.
- **2026-04-25 — Phase 2 map + movement:** strategic graph, deterministic pathfinding, movement orders/events, validator + resolver, `gc1805 move-all-to-capital` CLI subcommand.
- **2026-04-25 — Phase 3 economy:** economic phase resolver, economic order vocabulary, replacement/production/subsidy handling, `gc1805 economic-phase` CLI subcommand.
- **2026-04-25 — Phase 4 land combat:** battle outcome types, combat events, ZoC helpers, attack/bombard orders, resolver skeleton, combat table stubs, combat rules doc.
- **2026-04-25 — Phase 5 supply:** supply tracing, attrition integration, and supporting tests on branch `phase5-supply`.
- **2026-04-25 — Phase 6 diplomacy:** diplomacy orders, resolver work, and supporting tests on branch `phase6-diplomacy`.
- **2026-04-25 — Phase 7 political:** political points, revolts, abdication/state-change logic, and supporting tests on branch `phase7-political`.
- **2026-04-25 — Phase 8 minors:** tracked as a roadmap phase; release gate remains open pending the definitive minor-country list (Q6).
- **2026-04-25 — Phase 9 naval:** sea graph, fleet movement, naval combat, and transport support on branch `phase9-naval`.
- **2026-04-25 — Phase 10 full turn loop:** orchestrator/all-tables integration and determinism-focused tests on branch `phase10-turn-loop`.
- **2026-04-25 — Phase 11 AI:** roadmap phase reserved for deterministic, non-cheating AI integration; release readiness still depends on integration verification.
- **2026-04-25 — Phase 12 server:** axum HTTP/WebSocket server, game sessions, order submission, reconnect handling on branch `phase12-server`.
- **2026-04-25 — Phase 13 PBEM:** signed PBEM envelopes and host collection flow on branch `phase13-pbem`.
- **2026-04-25 — Phase 14 desktop UI:** Bevy app skeleton, UI state machine, and area-node work on branch `phase14-desktop-ui`.
- **2026-04-25 — Phase 15 web UI:** WASM bridge and React/web skeleton on branch `phase15-web-ui`.
- **2026-04-25 — Phase 16 localization:** English source locale, pseudo-locale, placeholder non-English locale set, locale loader on branch `phase16-localization`.
- **2026-04-25 — Phase 17 modding:** mod loader, override-resolution plumbing, and example mod support on branch `phase17-modding`.
- **2026-04-25 — Phase 18 replay:** event-log fold, seek-to-turn, replay integrity checks on branch `phase18-replay`.
- **2026-04-25 — Phase 19 polish / closed-beta prep:** CLI smoke test, release checklist, architecture doc, README cleanup, changelog normalization, and phase report.

### Changed

- Project documentation now explicitly distinguishes **structural phase completion** from **public-release readiness**.
- README now documents current build, test, lint, and CLI usage for the workspace.
- Phase 19 adds a `gc1805 smoke-test` command for quick integration validation against the standard scenario.

### Rules

- Schema version 1 remains in effect.
- Designer-authored table values may still be `PLACEHOLDER`; the scenario therefore remains `unplayable_in_release: true` until the open design questions are resolved.

### Fixed

- Repository-level release-prep docs now call out public-release blockers directly: rules tables, minors list, translators, hardware/perf sign-off, license, WASM verification, CI matrix verification, and toolchain pinning.
