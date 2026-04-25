# Architectural Decision Records

## ADR 0001: Initial §23 answers and contradiction resolution

Date: 2026-04-24
Status: **Provisional — awaiting designer confirmation**

### Context

Two inputs arrived in the same session:

1. A design-bundle prototype (`reference/prototype/`) titled *The Dusk of the
   Old World* — an original Napoleonic concept in HTML/CSS/JS.
2. A master prompt (`docs/PROMPT.md`) for a clean-room Rust/Bevy
   reimplementation, working title *Grand Campaign 1805*.

The prompt's §0.1 forbids silently overriding it; §23 forbids starting Phase 0
without answers to eight questions. The user instructed Claude Code to
"choose for me." These are those choices.

### Decisions

**Contradiction — prototype vs. master prompt.** The master prompt wins for
the live project. The prototype is preserved under `reference/prototype/` as
a visual style reference for Phase 14 (desktop UI). It is explicitly **not**
authoritative for rules, data, or architecture.

**§23.1 — Legal posture.** Posture **C (clean-room)**. No *Empires in Arms*
trademarks, art, or verbatim rules text. This matches the prototype chat's
own stance ("not branded as EiA").

**§23.2 — License / repo visibility.** Repo remains private. `LICENSE` file
is intentionally not created; defaults to all-rights-reserved. Revisit before
any public release.

**§23.3 — Reference hardware.** Ryzen 5 5600 / 16 GB RAM / iGPU-class (RX Vega
or Intel Xe) / 1080p. Performance targets in §17.7 are measured against this.

**§23.4 — Designer cadence.** **Unresolved.** A human designer is required
per §6.1 to author all values in `/data/tables/`. Weekly sync is the
placeholder cadence. See `questions.md` Q1.

**§23.5 — Determinism seed.** `0x5EEDED0000000001` (prompt default).

**§23.6 — §1.5 non-goals.** Unchanged.

**§23.7 — Continental System (§7.11).** **Out of v1.0.** Optional by the
prompt; can be added as a post-launch data pack.

**§23.8 — Named events (§7.8).** **In v1.0.** The closed effect vocabulary
keeps scripting safe and data-driven, and scenarios need it for period color.

### Consequences

- The four prototype files are git-moved to `reference/prototype/` — no
  history loss.
- Phase 0 scaffolding can begin immediately; Phase 1 cannot complete until
  a designer is assigned (blocks `/data/scenarios/1805_standard/` and
  `/data/tables/*.json` content).
- Continental System code paths are scoped out. A single feature flag
  `features.continental_system = false` will sit in scenario schema as an
  affordance for the eventual data pack.
- No `LICENSE` file ships until the user decides. Any external contribution
  before that is blocked by default copyright.

### Open items for human sign-off

Every decision above is reversible. See `questions.md` for the items that
require a human answer before they can be closed.


## 2026-04-25 — Q1 and Q9 resolved

- **Q1 (combat tables):** Closed. The designer-authored `combat.json` from `/tmp/q_answers/Q1_combat.json` replaced the placeholder table, and `gc1805-core-schema` / `gc1805-core` were updated to consume the real ratio-bucket / formation / results-table structure directly instead of the temporary Phase 4 placeholder schema (commit `eab2577`).
- **Q9 (Rust toolchain):** Closed. ADR 0001 accepted Rust `1.95.0` as the pinned stable toolchain. The pin now lives in `rust-toolchain.toml`, `workspace.package.rust-version`, and CI, with an explicit verification step to fail on toolchain drift (commit `eab2577`).
