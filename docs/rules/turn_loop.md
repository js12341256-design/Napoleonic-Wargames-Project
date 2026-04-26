# Full Turn Loop — canonical reference

This document defines the deterministic six-stage turn structure used by
`gc1805-core::turn_loop::run_turn`.

## Turn order

A campaign turn runs in this fixed order:

1. **TurnStarted** — emit `Event::TurnStarted { turn }`.
2. **Economic** — apply queued economic orders, then resolve the economic
   phase (`income -> maintenance -> replacements -> production -> subsidies`).
3. **Movement** — validate and resolve each movement-family order in the
   caller-provided order list.
4. **Combat** — validate and resolve each attack order in deterministic list
   order.
5. **Supply** — reserved stub phase for future supply / attrition wiring.
6. **Political** — reserved stub phase for diplomacy / political state.
7. **TurnCompleted** — increment `scenario.current_turn`, compute canonical
   state hash, emit `Event::TurnCompleted { turn, state_hash }`.

## Phase completion events

After each phase, emit:

- `Event::PhaseCompleted { turn, phase_name: "ECONOMIC" }`
- `Event::PhaseCompleted { turn, phase_name: "MOVEMENT" }`
- `Event::PhaseCompleted { turn, phase_name: "COMBAT" }`
- `Event::PhaseCompleted { turn, phase_name: "SUPPLY" }`
- `Event::PhaseCompleted { turn, phase_name: "POLITICAL" }`

The `turn` field is always the original turn number, not the incremented one.

## Determinism rules

- No floats.
- No `HashMap` in simulation logic; iteration is deterministic.
- Order lists are resolved in caller-provided sequence.
- Combat seed use is deterministic; multiple battles in one turn derive from
  `rng_seed` by stable index.
- Final state hash is produced from the canonical scenario representation.

## Current scope

Phase 10 wires the orchestration layer only.  Supply and political remain
stubbed with completion events so the outer turn contract is already stable.
