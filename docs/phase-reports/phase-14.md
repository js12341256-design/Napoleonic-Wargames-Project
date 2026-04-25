# Phase 14 Report — Desktop UI (Bevy)

## Summary

Phase 14 replaces the desktop client stub with a Bevy 0.15 application skeleton.

## Delivered

- Added Bevy 0.15 desktop dependencies to `gc1805-client-desktop`.
- Added `app_state.rs` with an application-state vocabulary, transition helper, display-name helper, and 11 non-Bevy tests.
- Replaced the desktop stub binary with a Bevy app that:
  - opens a `1280×720` window titled `Grand Campaign 1805`
  - spawns a 2D camera
  - renders a centered main menu
  - transitions to a game-board placeholder when `SPACE` is pressed
- Added desktop UI documentation in `docs/rules/ui_desktop.md`.

## Validation

Required validation for this phase:

- `cargo build -p gc1805-client-desktop`
- `cargo test -p gc1805-client-desktop`

## Notes

- The game-board screen is intentionally a placeholder only.
- Non-visual logic coverage for the phase lives in `app_state.rs` tests so the phase includes pure Rust validation independent of Bevy rendering.
