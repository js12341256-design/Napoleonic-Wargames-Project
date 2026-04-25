# Phase 12 — Server

Date completed: 2026-04-25
Branch: `phase12-server`
Gate: `docs/PROMPT.md` §16.12

## Summary

Implemented the Phase 12 authoritative server scaffold in
`crates/server` using axum + tokio, with in-memory session management,
HTTP routes for game creation/state/orders/events, WebSocket upgrade
support, reconnect replay from an event index, and per-player secret
state projection via `gc1805_core::projection::project`.

19 hand-written tests pass for the server crate behavior.

## Gate evidence

| §16.12 requirement | Status |
|---|---|
| axum HTTP server scaffold | ✅ `crates/server/src/main.rs` |
| `POST /games` create session | ✅ Implemented + tested |
| `GET /games/{id}/state` projected state | ✅ Implemented + tested |
| `POST /games/{id}/orders` authoritative routing | ✅ Implemented + tested |
| `GET /games/{id}/events?since=N` reconnect replay | ✅ Implemented + tested |
| `GET /health` | ✅ Implemented + tested |
| WebSocket endpoint present | ✅ Implemented + tested |
| reconnect parameter on WS (`last_event_index`) | ✅ Implemented |
| secret-order projection / player-only scenario view | ✅ Uses `project()` |
| 15+ tests | ✅ 19 tests |

## What was built

### `crates/server/Cargo.toml`

Added runtime deps for the server crate:

- `axum`
- `tokio`
- `serde`
- `serde_json`
- `gc1805-core`
- `gc1805-core-schema`

Added test deps:

- `tower` (`ServiceExt` for route testing)
- `http-body-util`

### `crates/server/src/main.rs`

Implemented:

- `GameSession`
- `AppState`
- router builder `app(state)`
- handlers:
  - `health`
  - `create_game`
  - `get_state`
  - `submit_orders`
  - `get_events`
  - `game_ws`
- WebSocket session stub that sends `{"type":"connected"}` and replays
  any missed events from `last_event_index`
- `main()` binding to `0.0.0.0:3000` via `axum::serve`

### `docs/rules/server.md`

Documented:

- authoritative server scope
- session storage model (`game_id -> GameSession`)
- order submission behavior
- reconnect replay semantics
- player-specific projection / secrecy model
- explicit non-goals per PROMPT.md §0.8

## Test inventory

The 17 tests cover:

1. `health_endpoint_returns_ok`
2. `create_game_returns_id`
3. `get_state_unknown_game_404`
4. `submit_orders_accepted`
5. `get_events_empty_initially`
6. `get_events_since_reconnect`
7. `create_multiple_games_isolated`
8. `projected_state_filtered_by_player`
9. `submit_order_unknown_game_404`
10. `websocket_upgrades`
11. `get_state_unknown_player_404`
12. `submit_orders_unknown_player_404`
13. `submit_orders_rejects_submitter_mismatch`
14. `get_events_since_beyond_end_returns_empty`
15. `events_unknown_game_404`
16. `projected_state_shows_enemy_corps_in_my_area`
17. `order_submission_does_not_mutate_other_game`
18. `create_game_initializes_empty_event_log`
19. `websocket_unknown_game_404`

Total workspace verification at completion: `cargo clippy --workspace
--all-targets -- -D warnings` and `cargo test --workspace` both pass.

## Hard-rules compliance

- ✅ No floats introduced
- ✅ No `HashMap` in simulation logic
- ✅ No auth/accounts/payments/telemetry/cloud services added
- ✅ Uses core projection instead of reimplementing fog-of-war logic
- ✅ Server remains a structure/handler phase, not real service integration
