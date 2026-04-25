# Web UI rules reference

Phase gate: `docs/PROMPT.md` §16.15 (Web UI — Rust/WASM + React).

## Scope

Phase 15 introduces the browser-facing shell for **Grand Campaign 1805**:

- a Rust → WASM bridge built with `wasm-bindgen` in `crates/client-web`
- a TypeScript/React shell in `/web`
- a browser presentation layer that will ultimately render the strategic map
- the integration seam for the multiplayer server delivered in Phase 12

This phase is intentionally a **skeleton**. Full map polish, final art, and richer browser presentation remain later work; Phase 19 is where PixiJS rendering grows beyond the scaffold.

## Rust → WASM bridge

`crates/client-web` is the narrow web boundary around deterministic core logic. The bridge:

- accepts scenario JSON from JavaScript
- deserializes it into `gc1805_core_schema::scenario::Scenario`
- exposes safe browser-callable methods through `wasm-bindgen`
- serializes results back to JSON strings for the React shell

The current Phase 15 bridge centers on `WasmGame`, which provides:

- scenario initialization from JSON
- current-turn lookup
- treasury lookup by power ID
- serialized power-ID and area-ID accessors
- `run_economic_phase` as the first browser-callable mutating game action

This bridge is the foundation for the Phase 15 export surface requested in `PROMPT.md` §16.15, including the browser-side scenario initialization path and later-order / projected-state plumbing.

## Browser shell (`/web`)

The `/web` directory hosts the TypeScript/React shell. In this phase it is responsible for:

- bootstrapping the browser app with Vite
- owning top-level loading / idle / playing UI state
- providing a stable mount point for future WASM loading
- listing basic game information without attempting final UX polish

The shell is intentionally minimal and avoids pretending that Phase 19 rendering already exists.

## Rendering plan: PixiJS

PROMPT.md names PixiJS in the web stack. For Phase 15, the rendering rule is:

- prepare the web shell so browser rendering can be attached cleanly
- treat PixiJS as the planned map-rendering layer
- defer real sprite work, map polish, and production presentation to Phase 19

In short: Phase 15 proves the shell and the bridge, not the finished battlefield map.

## Expected browser exports / interactions

The Phase 15 browser architecture must leave room for these web-facing actions:

- initialize a scenario in the browser
- fetch projected state for UI consumption
- submit player orders
- run the economic phase

`run_economic_phase` is implemented directly in the Rust/WASM bridge in this phase. The remaining higher-level browser workflows continue to build on the same bridge boundary in later phases instead of bypassing it.

## Networking seam

The Web UI must connect to the multiplayer service introduced in Phase 12 through a browser-safe transport. The intended path is a **WebSocket** connection from the React shell to the Phase 12 server.

Phase 15 establishes the client-side shell and boundary where that connection will live; it does not replace the deterministic headless core.

## Non-goals for this phase

- no invented gameplay values
- no replacement of deterministic Rust core logic with JavaScript logic
- no final PixiJS art / sprites / polished map interaction
- no scope creep into Phase 19 UI fidelity work
