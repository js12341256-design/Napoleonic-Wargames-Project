# PBEM (Play By Email)

Phase 13 implements the envelope/signing layer for PBEM play described in
`docs/PROMPT.md` §10 and gated by §16.13.

## Scope

This phase provides **signed order envelopes only**.  It does **not** send
email, open sockets, or perform any real-world networking.  Per `PROMPT.md`
§0 and §10, PBEM is host-mediated: players prepare orders offline, the host
collects them, verifies them, resolves the turn deterministically, and then
redistributes the results.

## Order envelope

Each player's turn submission is wrapped in an Ed25519-signed envelope:

- `player_id` — stable player/power identifier for the submitting player
- `game_id` — stable campaign identifier
- `turn` — logical turn number being submitted
- `orders_json` — canonical JSON payload for that player's `Vec<Order>`
- `signature` — Ed25519 signature over the exact message
  `player_id:game_id:turn:orders_json`

The signed message format is deliberately simple and deterministic.  Any change
in player, game, turn, or canonical order payload invalidates the signature.

## Verification flow

Host-side PBEM intake follows this sequence:

1. Receive an `OrderEnvelope` from a player by any out-of-band transport.
2. Reconstruct the signed message as
   `player_id:game_id:turn:orders_json`.
3. Verify the Ed25519 signature using the player's public key.
4. Reject the envelope if:
   - the signature is invalid,
   - the envelope turn does not match the host's current turn,
   - the envelope game id does not match the host game,
   - that player already submitted an envelope for the turn.
5. Store the verified envelope in host collection state.

This keeps PBEM deterministic and auditable while avoiding any dependence on
transport details.

## Host-mediated turn resolution

Per `PROMPT.md` §10 and §16.13, PBEM turn resolution is host mediated:

1. The host waits until all expected players have submitted envelopes.
2. The host verifies every envelope before any resolution runs.
3. The host extracts the canonical order payloads.
4. The deterministic resolver executes the turn exactly once.
5. The host distributes the resulting state/events/save artifacts back to the
   players using whatever manual or future transport layer is chosen.

Phase 13 stops at the envelope/signature layer so later phases can choose a
transport without changing the signed payload contract.
