# Server rules — canonical reference

Sourced from PROMPT.md §9 (multiplayer specifics), §16.12 (server gate),
and §0.8 (no real networking/accounts/payments beyond the explicitly
requested local server structure).

## 1. Scope

Phase 12 provides the authoritative in-memory server shell for Grand
Campaign 1805:

- axum HTTP server for game/session lifecycle and state fetches
- axum WebSocket endpoint for live session attachment
- deterministic session storage keyed by `game_id`
- reconnect support via event-log replay from a client-supplied index
- secret-state projection so each player only receives that player's
  `ProjectedScenario`

This phase does **not** add external accounts, telemetry, payment flows,
cloud services, or any other real-world integration forbidden by
PROMPT.md §0.8.

## 2. Session model

Server state is:

- `AppState.sessions: Mutex<BTreeMap<String, GameSession>>`

Each `GameSession` stores:

- `game_id`
- authoritative full `Scenario`
- append-only `event_log: Vec<Event>`
- `players: BTreeMap<String, PowerId>` mapping `player_id -> power`

`BTreeMap` is used for deterministic ordering.  Simulation state remains
in the core/schema crates; the server only routes requests, stores the
current session snapshot, and exposes player-filtered views.

## 3. HTTP routes

### `POST /games`

Creates a new in-memory `GameSession` and returns a generated `game_id`.
The initial scenario is an empty placeholder session scaffold suitable
for tests and future phase integration.

### `GET /games/{game_id}/state?player_id=...`

Returns the requesting player's projected scenario.

Flow:

1. Find `GameSession` by `game_id`
2. Resolve `player_id -> PowerId`
3. Run `gc1805_core::projection::project(&scenario, &power)`
4. Return `ProjectedScenario.view`

This is the core multiplayer secrecy rule from PROMPT.md §9: the server
keeps the authoritative full state, while clients receive only their own
filtered projection.

### `POST /games/{game_id}/orders`

Accepts JSON:

```json
{
  "player_id": "player-fra",
  "orders": [ ... ]
}
```

Phase 12 behavior:

- verifies the game exists
- verifies the player exists in the session
- verifies every order submitter matches the power controlled by that
  player
- returns `{ "accepted": N, "event_log_len": M }`

This phase intentionally stops at authoritative routing/validation.
Actual adjudication and event emission stay in later gameplay phases.

## 4. Reconnect support

### HTTP replay

`GET /games/{game_id}/events?since=N`

Returns `event_log[N..]`.

This is the reconnect primitive from PROMPT.md §9: when a client drops,
it can reconnect with its last known event index and replay the missing
suffix deterministically from the authoritative log.

### WebSocket replay hint

`GET /games/{game_id}/ws?last_event_index=N`

On upgrade, the server:

1. accepts the WebSocket
2. sends `{"type":"connected"}`
3. replays `event_log[N..]` as JSON messages
4. idles waiting for future work

The live push stream is currently a stub by design: the connection shape
exists, replay works, and later phases can attach real event broadcast.

## 5. Health route

`GET /health` returns:

```json
{"status":"ok"}
```

This is intentionally tiny and side-effect free.

## 6. Security / non-goals for this phase

Per PROMPT.md §0.8 and §9:

- no auth provider integration
- no account system
- no payment system
- no telemetry or analytics
- no external network dependencies beyond binding the local axum server
- no simulation logic inside server handlers

## 7. Test coverage expectations

Phase 12 requires 15+ tests.  Coverage includes:

- health route
- game creation
- unknown-game handling
- player projection filtering
- order routing acceptance/rejection
- event replay semantics (`since` reconnect behavior)
- isolation between multiple games
- basic WebSocket upgrade behavior

The goal of the phase is not full netcode sophistication; it is a clean,
testable authoritative server structure aligned with PROMPT.md §9 and
§16.12.
