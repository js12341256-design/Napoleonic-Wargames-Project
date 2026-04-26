# Architecture

This document is a high-level map of the Grand Campaign 1805 codebase as it stands during the Phase 19 polish pass.

Where a subsystem is only present on a dedicated phase branch, that is called out explicitly. The goal here is to describe the intended architecture without pretending placeholder data or not-yet-integrated branches are already public-release complete.

## 1. Crate dependency graph

Workspace crates visible from the root `Cargo.toml`:

```text
gc1805-core-schema
    └─ shared type definitions, canonical JSON, IDs, events, scenario/tables schemas

gc1805-core
    ├─ depends on gc1805-core-schema
    └─ owns deterministic simulation logic (loader, validation, movement, economy, combat)

gc1805-core-validate
    ├─ depends on gc1805-core
    └─ exposes stable validation entry points for clients/servers

gc1805-core-rng
    └─ deterministic RNG utilities (kept separate from rules code)

gc1805-ai
    ├─ intended to depend on gc1805-core / projections
    └─ deterministic non-cheating AI layer

gc1805-netcode
    └─ protocol and message types for multiplayer transport

gc1805-client-shared
    └─ logic shared by desktop and web front ends

gc1805-server
    ├─ intended axum/tokio authoritative host
    ├─ uses netcode-facing message types
    └─ calls into validation / simulation layers

gc1805-client-desktop
    ├─ intended Bevy desktop shell
    └─ consumes client-shared + netcode + projections

gc1805-client-web
    ├─ intended WASM bridge
    └─ sits beside the `/web` React shell

gc1805-cli
    ├─ depends on gc1805-core
    ├─ depends on gc1805-core-schema
    └─ depends on gc1805-core-validate

xtask
    └─ maintenance / codegen / schema dumping helpers

asset-pipeline
    └─ offline content processing (sprites, locales, map assets)
```

### Practical dependency rule

The simulation heart is:

```text
gc1805-core-schema -> gc1805-core -> gc1805-core-validate -> {CLI/server/clients/AI}
```

That split matters because:

- schemas stay serializable and stable,
- deterministic rules code stays headless,
- validation can be reused by multiple front ends,
- and UI/network layers do not get to invent rules behavior.

## 2. Data flow

The intended data flow is event-sourced and deterministic.

```text
scenario.json
  -> loader
  -> typed Scenario
  -> order submission / validation
  -> resolver
  -> ordered Event log
  -> folded state update
  -> new Scenario / saved game state
```

In more detail:

1. **Scenario load**
   - `data/scenarios/1805_standard/scenario.json` is read.
   - `gc1805-core::load_scenario_str` parses JSON, checks `schema_version`, scans for `PLACEHOLDER` markers, initializes missing live `power_state`, and runs structural validation.

2. **Orders**
   - Players, AI, PBEM envelopes, or CLI helpers produce typed `Order` values.
   - `gc1805-core-validate` and `gc1805-core` validators confirm the order is legal for the current state.

3. **Resolution**
   - A phase-specific resolver produces ordered `Event` values.
   - Those events are deterministic consequences of the current state plus the submitted order set.

4. **State advancement**
   - Event effects mutate the working scenario or are later replay-folded into a derived state.
   - The resulting scenario becomes the input to the next phase or turn.

5. **Persistence / replay / PBEM**
   - Saves, replays, and PBEM all depend on the same stable typed data and event representation.

## 3. Six-phase turn pipeline

The phase branches and prompt describe a six-phase monthly turn loop. At a high level:

1. **Economic phase**
   - Income
   - Maintenance
   - Replacements
   - Production
   - Subsidies

2. **Diplomatic / political phase**
   - Diplomatic actions
   - Treaty-state changes
   - Political points, revolts, and state shifts

3. **Movement / supply phase**
   - Strategic movement orders
   - Forced march handling
   - Supply tracing / attrition
   - Interception setup where applicable

4. **Combat phase**
   - Land combat
   - Retreat / rout / casualties
   - Naval actions on the naval branch

5. **Administrative / reinforcement phase**
   - Corps/fleet arrival timing
   - Replacements and queued effects that mature by turn index
   - Scenario cleanup for next turn

6. **Turn advance / replayable finalization**
   - Deterministic event log finalization
   - Hashing / save / PBEM envelope production
   - Increment `current_turn`

Exact sub-step ownership depends on the phase branches, but the architectural rule is simple: all turn progression flows through deterministic ordered state transitions, not ad hoc UI-side mutations.

## 4. Multiplayer architecture

## Server

The multiplayer server architecture is intended to be authoritative:

- **Transport:** axum HTTP + WebSocket
- **Concurrency/runtime:** tokio
- **Role:** accept sessions, validate orders, serialize authoritative event/state updates, and manage reconnects

The remote `phase12-server` branch records that server structure exists at the branch level.

## WebSocket play

For live multiplayer:

```text
client
  -> submit order/message
  -> server validates against authoritative Scenario
  -> server resolves accepted actions
  -> server emits ordered updates/events
  -> clients project/render visible state
```

Key principle: clients are views and order-entry tools, not rules authorities.

## PBEM

The remote `phase13-pbem` branch adds PBEM envelopes. Architecturally that means:

- orders are serialized into signed envelopes,
- the host collects them,
- deterministic resolution happens from the same shared rules code,
- and replay / audit can inspect the envelope chain and event results.

PBEM therefore shares the same core state machine as live multiplayer; only transport and timing differ.

## 5. UI layers

The project has three intended user-facing surfaces:

### Headless CLI

Current branch reality:

- `gc1805 load`
- `gc1805 move-all-to-capital`
- `gc1805 economic-phase`
- `gc1805 smoke-test`

The CLI is the first integration surface because it proves the core can run without graphics, accounts, or networking.

### Desktop UI

Intended architecture:

- Bevy-based desktop application
- consumes projected scenario data and shared client logic
- renders strategic map, orders, and replay state

Repository evidence:

- dedicated remote branch: `phase14-desktop-ui`

### Web UI

Intended architecture:

- Rust/WASM bridge in `crates/client-web`
- React shell under `/web`
- same core game concepts as desktop, different presentation/runtime layer

Repository evidence:

- dedicated remote branch: `phase15-web-ui`

The release checklist keeps WASM verification explicitly open because branch existence is not the same thing as a verified release build.

## 6. Placeholder / `Maybe<T>` system

A central architectural choice is the explicit placeholder system in `gc1805-core-schema::tables::Maybe<T>`.

```rust
pub enum Maybe<T> {
    Value(T),
    Placeholder(PlaceholderMarker),
}
```

This does three jobs:

1. **Prevents invented values**
   - designer-authored numerics do not get guessed in code.

2. **Allows structural progress**
   - phases can build schema, order, resolver, and integration plumbing before tables are authored.

3. **Gates release honestly**
   - loaders surface placeholder paths,
   - scenarios can remain `unplayable_in_release: true`,
   - and docs/checklists can say exactly why a branch is structurally complete but not playable.

### Why this matters

Without `Maybe<T>`, placeholder-era code would either:

- hardcode fake values, which violates the prompt, or
- fail to load at all, which blocks structural integration work.

`Maybe<T>` is the compromise that keeps the architecture honest.

## 7. Determinism constraints that shape the architecture

Several architecture decisions are there specifically to preserve deterministic simulation:

- `BTreeMap` / `BTreeSet` instead of hash-order iteration for stateful logic
- no floating-point in the simulation core
- no wall-clock time in rules code
- canonical JSON and stable IDs for persisted roots
- event-sourced resolution model
- headless execution before UI polish

This is why the project is split so aggressively between schema, simulation, validation, and presentation.

## 8. What Phase 19 adds architecturally

Phase 19 does not start a new subsystem. It tightens integration and release prep by:

- adding a CLI smoke test that exercises load + economy + movement validation,
- documenting the release gates and open blockers,
- documenting the architecture in one place,
- and making the repository status legible for closed-beta preparation.
