# Phase 0 — Scaffolding

Date closed: 2026-04-24
Branch: `claude/implement-design-system-ciX1R`
Gate: `docs/PROMPT.md` §16.1

## Summary

Project initialized.  Workspace builds clean, tests run with zero tests as
the gate specifies, clippy and rustfmt pass.  Thirteen crate skeletons are
in place, directory layout matches §4 exactly, baseline documentation is
authored, and a CI workflow is ready for the repo's GitHub Actions runner.

The prototype from the *Dusk of the Old World* design bundle has been
archived under `reference/prototype/` as a visual reference only.

## Gate evidence

| §16.1 requirement                          | Status                                              |
|--------------------------------------------|-----------------------------------------------------|
| `cargo build --workspace` succeeds         | ✅ Linux local (0.66 s).  macOS / Windows in CI.    |
| `cargo test --workspace` runs (0 tests)    | ✅ 13 crates × 0 tests, all green.                  |
| CI pipeline on PR                          | ✅ `.github/workflows/ci.yml` — fmt, clippy, build+test matrix (Linux/macOS/Windows), web-lint placeholder. |
| Repository layout matches §4               | ✅ Every path in §4 exists.  `LICENSE` deliberately absent per ADR 0001.                               |
| `docs/SPEC.md`, `PROMPT.md`, `questions.md`, `ideas.md`, `decisions.md` exist | ✅ All present.  `SPEC.md` is a placeholder; see `questions.md` Q2. |

Cross-platform verification from §16.1 ("on all three platforms") is
delegated to the CI matrix; only Linux x86_64 has been verified locally.

## Deliverables in this phase

### New files (top level)

- `Cargo.toml` — workspace root with 13 members, resolver v2, release
  profile with `lto = "thin"` and `overflow-checks = true`.
- `rust-toolchain.toml` — pinned to stable 1.94.1 with rustfmt, clippy,
  and `wasm32-unknown-unknown` target.
- `.nvmrc` — Node 22.
- `.gitignore` — ignores `target/`, `node_modules/`, editor droppings,
  asset-pipeline build caches.
- `README.md` — overview, repository map, ground rules, license note.
- `CHANGELOG.md` — Keep-a-Changelog format with an Unreleased section.

### Crate stubs

Every crate has `Cargo.toml` + `src/lib.rs` or `src/main.rs`:

- `gc1805-core`, `gc1805-core-schema`, `gc1805-core-rng`,
  `gc1805-core-validate`, `gc1805-ai`, `gc1805-netcode`,
  `gc1805-client-shared`, `gc1805-client-web` — libraries.
- `gc1805-server`, `gc1805-client-desktop`, `gc1805-cli`,
  `xtask`, `asset-pipeline` — binaries that print "Phase 0 stub" and exit.

All library crates declare `#![forbid(unsafe_code)]`.  `gc1805-core`
additionally declares `#![deny(clippy::float_arithmetic)]` per §2.2
(no floats in the simulation core).

No external crate dependencies are declared yet.  Every dep in
`docs/deps.md` will be introduced in the phase that first requires it, to
keep Phase 0 build time under a second and to avoid importing code that
might fail gates we haven't written yet.

### Documentation

Already present (authored in the initial scaffolding pass):

- `docs/PROMPT.md` — verbatim copy of the master prompt.
- `docs/SPEC.md` — placeholder; §5/§11.2/§21 contents pending.
  See `questions.md` Q2.
- `docs/decisions.md` — ADR 0001 records all eight §23 choices and the
  prototype-vs-master-prompt contradiction resolution.
- `docs/questions.md` — Q1–Q8 open for the designer.
- `docs/ideas.md`, `docs/adjudications.md` — empty parking lots.
- `docs/deps.md` — dependency justifications per §3.1.
- `docs/rules/README.md` — index of the per-subsystem rules files to come.

Added in this phase:

- `docs/phase-reports/phase-00.md` (this file).
- `.github/workflows/ci.yml` — fmt + clippy + cross-platform build/test
  + web-lint placeholder.
- `web/package.json` + empty `src/` and `public/` dirs (Phase 15 stake).
- `.gitkeep` markers in every §4 directory that would otherwise be empty
  (`data/{scenarios,tables,map}`, `data/ai/personalities`, `locales`,
  `testdata/{rules_cases,scenarios,saves}`, `assets/{fonts,sprites,audio,ui}`).

### Prototype archive

`reference/prototype/` holds the HTML/CSS/JS design bundle (`index.html`,
`styles.css`, `data.js`, `app.js`, `README.md`) via `git mv`, so history is
preserved.  Its README explicitly flags it as visual-only, non-authoritative.

## ADRs added this phase

- **ADR 0001** — §23 answers + prototype-vs-master-prompt resolution.

## Adjudications added this phase

None.  `docs/adjudications.md` is empty until Phase 4+.

## Open questions (parking lot for the designer)

All eight entries in `docs/questions.md` are open:

- **Q1** — Designer for rules tables (blocks Phase 1 gate onward).
- **Q2** — `docs/SPEC.md` contents (blocks Phase 1, 6, 14 gates).
- **Q3** — CI host (default GitHub Actions assumed).
- **Q4** — License decision.
- **Q5** — Tutorial scenario design.
- **Q6** — 1805 minor-country definitive list.
- **Q7** — Locale translators.
- **Q8** — Reference-hardware access for benchmarks.

## Known defects and caveats

- CI workflow has only been authored, not executed.  Its matrix rows for
  macOS and Windows need a first run to validate toolchain-install steps.
- The §2.7 weekly cross-platform determinism job is **not** wired up yet.
  It can't run before Phase 1 (there is no state to hash).  Added to the
  Phase 1 opening checklist rather than a separate question.
- `Cargo.lock` is not present.  With zero external deps there is nothing
  to lock; it will appear in Phase 1 when `serde` etc. are introduced.
- No license scanner in CI.  Trivially unnecessary at Phase 0 (no third-
  party source); will be added when the first deps land.

## Next phase

Phase 1 — Data model and scenario loader (`docs/PROMPT.md` §16.2).  Do
**not** start until Q1 and Q2 are answered (or the user explicitly opts in
to treating PROMPT.md as the complete spec and accepts that the 1805
scenario will ship with PLACEHOLDER values until a designer is assigned).
