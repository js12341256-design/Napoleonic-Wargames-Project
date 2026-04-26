# Phase 19 — Polish / closed-beta preparation

Date closed: 2026-04-25
Branch: `phase19-polish`
Gate: `docs/PROMPT.md` §16.19 and §22

## Summary

This phase is intentionally not a new subsystem. It is a quality, integration, and documentation pass intended to make the repository legible for closed-beta use.

Work completed in this phase:

- added a `gc1805 smoke-test` CLI subcommand,
- added `docs/RELEASE_CHECKLIST.md`,
- added `docs/ARCHITECTURE.md`,
- updated `README.md` to reflect the current build/test/CLI story,
- cleaned up `CHANGELOG.md` so every phase 0–19 is represented under `[Unreleased]`,
- and re-ran workspace build/test/clippy.

## Structural status

The project now has Phase 19 release-prep documentation in place. The repository also has dedicated remote branches for later subsystems including supply, diplomacy, political, naval, turn loop, server, PBEM, desktop UI, web UI, localization, modding, and replay.

For planning and closed-beta documentation purposes, all 19 phases are now accounted for in the roadmap and release checklist.

## Gates still open

The major open gates are unchanged from the earlier subsystem work:

- **Phase 4 — Land combat:** open pending **Q1**, because real combat-table values still need human authorship.
- **Phase 8 — Minors:** open pending **Q6**, because the full 1805 minor-country list is still missing.
- **Phase 16 — Localization:** open pending **Q7**, because human translators are still needed for FR / DE / ES / RU / PL / IT.

Additional release-level blockers remain outside those gate labels:

- **Q4** — license decision before any public release
- **Q8** — target hardware/performance sign-off
- **Q9** — real Rust version pin instead of floating `stable`

## Smoke-test coverage added

The new `gc1805 smoke-test` command now performs one narrow integration pass:

1. loads `data/scenarios/1805_standard/scenario.json`,
2. prints placeholder and integrity findings,
3. runs `resolve_economic_phase` with default economy tables,
4. validates a `Hold` movement order for the first corps in deterministic order,
5. prints `SMOKE TEST PASSED` on success.

This is not a gameplay certification. It is a quick regression tripwire for the closed-beta prep branch.

## What "closed beta" means here

For this project, **closed beta** does **not** mean "public playable release."

It means:

- the workspace builds, tests, and lints cleanly,
- core integrations can be sanity-checked from the CLI,
- documentation is honest about placeholder data,
- and trusted testers can exercise structure, workflows, and UX surfaces without pretending the rules tables are final.

Because designer-authored values still remain as `PLACEHOLDER` in multiple data files, the 1805 scenario still carries `unplayable_in_release: true`. That is the correct state until Q1, Q6, and Q7 are resolved and the remaining release blockers are signed off by humans.

## End-of-phase checklist snapshot

- [x] Workspace builds
- [x] Workspace tests pass
- [x] Workspace clippy passes with `-D warnings`
- [x] Release checklist authored
- [x] Architecture document authored
- [x] README updated
- [x] Changelog updated
- [x] Smoke-test CLI command added
- [ ] Public-release blockers resolved
- [ ] Placeholder data removed
- [ ] License decided
- [ ] WASM target verified on integration branch
- [ ] Cross-platform CI matrix signed off for release

## Notes for the next human pass

Before calling the project release-ready rather than closed-beta-ready:

1. resolve Q1 / Q6 / Q7,
2. choose and commit a license,
3. pin a real Rust toolchain version,
4. re-run CI across the expected platform matrix,
5. verify the web/WASM target on the integration branch,
6. and review the later phase branches for merge/integration sequencing.
