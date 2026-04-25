# Phase 15 — Web UI (Rust/WASM + React)

Date completed: 2026-04-25
Branch: `phase15-web-ui`
Gate: `docs/PROMPT.md` §16.15

## Summary

Phase 15 replaces the placeholder web client scaffold with a compiling Rust/WASM bridge in `crates/client-web` and a Vite/React shell in `web/`. The Rust side exposes browser-safe JSON-based scenario loading and lookup helpers plus an economic-phase bridge method. The React side is intentionally skeletal, matching the project requirement that full visual polish belongs to Phase 19.

## What changed

### Rust / WASM

- `crates/client-web/Cargo.toml`
  - switched library output to `cdylib` + `rlib`
  - added `wasm-bindgen`, `serde`, `serde_json`, `js-sys`, `console-error-panic-hook`
  - linked `gc1805-core` and `gc1805-core-schema`
- `crates/client-web/src/lib.rs`
  - added `#[wasm_bindgen(start)]` panic-hook initialization
  - implemented `WasmGame` scenario wrapper
  - exposed JSON-based accessors for turn, treasury, power IDs, area IDs, and economic-phase execution
  - added a native-testable helper module for parsing / serialization and ID lookups
  - added 12 native tests

### Web shell

- `web/package.json` now defines a Vite + React toolchain
- added `web/vite.config.ts`
- added `web/index.html`
- added `web/src/main.tsx`
- added `web/src/App.tsx`
- added `web/tsconfig.json`

### Documentation

- added `docs/rules/ui_web.md`
- updated `CHANGELOG.md`

## Validation

Commands run:

```sh
source ~/.cargo/env
cargo build -p gc1805-client-web
cargo test -p gc1805-client-web
```

Result: both commands pass after implementation fixes.

## Notes

- The bridge keeps deterministic simulation logic inside Rust.
- The React shell is a scaffold only; Phase 19 remains the target for real browser rendering polish.
- No gameplay values were invented for this phase.
