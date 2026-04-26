# Desktop UI Rules

Phase 14 introduces the desktop Bevy client skeleton for **Grand Campaign 1805**.

## Scope

- The desktop client opens a Bevy window titled `Grand Campaign 1805`.
- The initial UI flow is a main menu followed by a game-board placeholder screen.
- Input handling is limited to a single transition from the main menu to the game board when the player presses `SPACE`.
- Rules adjudication remains outside the desktop client. Simulation state continues to live in the core crates.

## Phase 14 Constraints

- Bevy version is `0.15`.
- The UI must compile in CI headless build contexts even though it renders a desktop window at runtime.
- Numerical rules data is not authored in this document and must not be duplicated here.
- The desktop client may depend on `gc1805-core` and `gc1805-core-schema`, but it must not redefine rule structures already owned by those crates.

## UI State Vocabulary

The UI state helper in `crates/client-desktop/src/app_state.rs` defines these application states:

- `MainMenu`
- `ScenarioSelect`
- `Loading`
- `GameBoard`
- `OrderEntry`
- `EndTurn`
- `Results`

Phase 14 only wires the runtime Bevy state machine for:

- `MainMenu`
- `GameBoard`

The remaining states are reserved for later phases and are covered by pure Rust tests in the helper module.

## Out of Scope

Phase 14 does **not** add:

- strategic map rendering
- scenario loading UX
- order entry widgets
- results screens
- localization assets
- persistence or PBEM flows

Those remain future-phase work per `docs/PROMPT.md`.
