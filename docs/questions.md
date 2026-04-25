# Open questions for the human designer

Claude Code must not guess answers here (see PROMPT.md §0.1, §6.1). Each
question blocks the phase named in its **Blocks** line. Close a question by
editing this file and referencing the commit that acts on the answer.

---

## Q1 — Who authors the rules tables?

**Status:** CLOSED — 2026-04-25. Designer-authored `data/tables/combat.json` integrated on branch `integrate/q1-combat-tables-q9-toolchain` (commit `eab2577`).

**Blocks:** Phase 1 gate (scenario completeness), Phase 4 and onward (every
rules subsystem).

Resolved for land combat: the human designer supplied the authored combat table at `/tmp/q_answers/Q1_combat.json`, and the Rust combat schema/resolver were updated to consume that real structure directly.

## Q2 — `docs/SPEC.md` location and contents

**Blocks:** Phase 1 gate (full data model), Phase 6 gate (diplomatic action
list), Phase 14 gate (list of screens).

PROMPT.md cites a separate design spec in §5, §11.2, §21 but it was not
included with the master prompt. Options:

- (a) Treat PROMPT.md as the complete spec; log every gap here as it's hit.
- (b) The designer produces `docs/SPEC.md` separately before Phase 1 starts.

Default chosen in ADR 0001: (a). Confirm or override.

## Q3 — CI host

**Blocks:** Phase 0 gate.

PROMPT.md §16.1 requires a CI pipeline. Options: GitHub Actions (default for
a GitHub-hosted repo), self-hosted runners, or a third-party CI (Buildkite,
CircleCI, etc.). The cross-platform matrix in §2.7 needs Linux x86_64,
Linux ARM64, macOS ARM64, and Windows x86_64.

Default proposed: GitHub Actions. Confirm.

## Q4 — License

**Blocks:** any public release or external contribution.

ADR 0001 leaves `LICENSE` unset (all-rights-reserved). Before any public
push, decide: proprietary / MIT / Apache-2.0 / GPL-3.0 / dual-licensed.

## Q5 — Tutorial design

**Blocks:** Phase 14 gate.

PROMPT.md §12 specifies a 2-player, 3-turn tutorial but leaves the scripted
outcomes to design. Needed: the scenario file, the coach overlay text, and
the highlight targets.

## Q6 — Minor-country list for 1805

**Blocks:** Phase 1 gate, Phase 8 gate.

PROMPT.md cites "approximately fifty minor countries." The definitive list
for the 1805 scenario is a design decision (which Imperial states are
distinct entities vs. bundled, how the Holy Roman Empire is modeled, etc.).

## Q7 — Locale translators

**Blocks:** Phase 16 gate.

Seven locales required. Claude Code can maintain `en.yaml` as the source and
produce the pseudo-locale `zz.yaml`, but human translators are needed for
French, German, Spanish, Russian, Polish, and Italian.

## Q8 — Reference-hardware access

**Blocks:** Phase 17.7 perf gates.

ADR 0001 picked a target spec but Claude Code has no machine of that spec
for benchmarking. Either the user runs the criterion benches on matching
hardware, or a CI runner of that spec is provisioned.

## Q9 — Rust toolchain version pin

**Status:** CLOSED — 2026-04-25. ADR accepted and implemented: Rust pinned to `1.95.0` in `rust-toolchain.toml`, workspace `Cargo.toml`, and CI (commit `eab2577`).

**Blocks:** PROMPT.md §3.6 compliance (pinned minor version, updated
quarterly).

Resolved via ADR 0001 (`/tmp/q_answers/Q9_ADR_0001_rust_toolchain.md`) and implementation in this branch. CI now verifies the active compiler matches the pinned toolchain exactly.
