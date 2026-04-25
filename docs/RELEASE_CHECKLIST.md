# Release Checklist

Phase 19 is a polish / closed-beta readiness pass. This checklist records what is structurally present, what is still blocked, and what still needs human sign-off before any public release.

## 1. Phase gates

Status meanings used here:

- **Closed** — structural phase work exists and the gate is treated as complete for closed-beta prep.
- **Open** — known blocker remains.
- **Open (verify on integration)** — branch-level work exists, but public-release verification still must be re-run on the integration branch.

| Phase | Area | Status | Notes |
|---|---|---:|---|
| 0 | Scaffolding | Closed | Workspace, CI skeleton, docs, crate layout. |
| 1 | Data model + loader | Closed | Scenario loader and schemas in place. |
| 2 | Map + movement | Closed | Pathing, movement orders, validation, CLI support. |
| 3 | Economy | Closed | Economic resolver and tests present. |
| 4 | Land combat | **Open** | Q1: real combat table values still required. |
| 5 | Supply | Closed | Remote branch `phase5-supply`. |
| 6 | Diplomacy | Closed | Remote branch `phase6-diplomacy`. |
| 7 | Political | Closed | Remote branch `phase7-political`. |
| 8 | Minors | **Open** | Full 1805 minor list still missing (Q6). |
| 9 | Naval | Closed | Remote branch `phase9-naval`. |
| 10 | Full turn loop | Closed | Remote branch `phase10-turn-loop`. |
| 11 | AI | Closed for roadmap tracking; verify on integration | Closed-beta checklist assumes structural completion, but integration evidence should be rechecked before public release. |
| 12 | Server | Closed | Remote branch `phase12-server`. |
| 13 | PBEM | Closed | Remote branch `phase13-pbem`. |
| 14 | Desktop UI | Closed | Remote branch `phase14-desktop-ui`. |
| 15 | Web UI / WASM | Open (verify on integration) | Remote branch `phase15-web-ui`; WASM target verification still required below. |
| 16 | Localization | **Open** | Q7: human translators still required for non-English locales. |
| 17 | Modding | Closed for structure; hardware/perf sign-off still open | Remote branch `phase17-modding`; Q8 still blocks perf validation. |
| 18 | Replay | Closed | Remote branch `phase18-replay`. |
| 19 | Polish / closed beta prep | Closed after this pass | Smoke test CLI, release checklist, architecture doc, README/changelog/report cleanup. |

## 2. Questions that must be resolved before release

All open questions in `docs/questions.md` remain release blockers unless explicitly marked otherwise.

| Question | Required before release? | Why |
|---|---:|---|
| Q1 — rules table authorship / real authored tables | Yes | Combat and other designer-authored tables cannot ship as placeholders. |
| Q2 — `docs/SPEC.md` completeness | Yes | Release docs should not depend on a missing in-session spec fragment. |
| Q3 — CI host confirmation | Yes | Release CI ownership and matrix expectations must be explicit. |
| Q4 — license | **Yes** | Public release cannot proceed without a license decision. |
| Q5 — tutorial design | Yes for public playable release | Tutorial is specified in the prompt and remains incomplete. |
| Q6 — full minor-country list | **Yes** | Phase 8 gate is open. |
| Q7 — locale translators | **Yes** | Phase 16 gate is open for FR/DE/ES/RU/PL/IT. |
| Q8 — reference hardware access | Yes | Performance gates cannot be signed off without target hardware. |
| Q9 — Rust toolchain pin | Yes | Current `stable` tracking violates the prompt's pinned-version requirement. |

## 3. Human sign-off required

These items require a human decision, review, or external asset source:

- **Combat tables** — real `data/tables/combat.json` values and related rules tables.
- **Minor-country list** — definitive 1805 minors model (Q6).
- **Translators** — FR / DE / ES / RU / PL / IT locale review and sign-off (Q7).
- **Reference hardware** — benchmark execution on the agreed target machine (Q8).
- **License** — explicit release license choice (Q4).

## 4. Public-release blockers

Before any public release, confirm all of the following are closed:

- [ ] Q1 resolved and placeholder combat / rules-table values replaced with human-authored data.
- [ ] Q4 resolved and `LICENSE` committed.
- [ ] Q6 resolved and Phase 8 minors data completed.
- [ ] Q7 resolved and non-English locales reviewed by humans.
- [ ] Q8 resolved and performance runs captured on reference hardware.
- [ ] Q9 resolved and the Rust toolchain is pinned to a real stable version.
- [ ] `docs/PROMPT.md` is restored in full, or an ADR explicitly states the authoritative substitute.

## 5. Closed-beta verification steps

These are acceptable for a closed beta even while placeholders remain, provided testers understand the build is not a public-playable release.

- [ ] `cargo build --workspace`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo run -p gc1805-cli -- smoke-test`
- [ ] Review smoke-test output for placeholder and integrity diagnostics.
- [ ] Confirm README, architecture doc, phase report, and changelog match repository reality.

## 6. WASM target verification

The web UI branch exists, but public release still requires a direct verification pass:

- [ ] `rustup target add wasm32-unknown-unknown`
- [ ] Build the web/WASM target successfully on the integration branch.
- [ ] Confirm any JS/React wrapper instructions in `/web` still match the Rust-side bridge.
- [ ] Record the result in the phase report or changelog.

## 7. Cross-platform CI matrix check

The prompt expects Linux x86_64, Linux ARM64, macOS ARM64, and Windows x86_64 coverage.

- [ ] Linux x86_64 build + test green.
- [ ] Linux ARM64 build + test green.
- [ ] macOS ARM64 build + test green.
- [ ] Windows x86_64 build + test green.
- [ ] Clippy and rustfmt gates green in CI.
- [ ] Any WASM/web job green.

## 8. Definition of ready for closed beta

This project is ready for a **closed beta** when:

- the workspace builds/tests/clippy cleanly,
- the smoke test passes,
- documentation accurately describes open placeholders and blocked gates,
- testers are explicitly told the scenario is still `unplayable_in_release: true`,
- and no one mistakes the build for a public-release rules-complete version.
