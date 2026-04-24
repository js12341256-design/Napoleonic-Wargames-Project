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
