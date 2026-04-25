# Supply rules — canonical reference

Sourced from `docs/PROMPT.md` §16.5.

## 1. Supply state

Per `PROMPT.md` §16.5, every corps is in exactly one supply state:

- `InSupply`
- `Foraging`
- `OutOfSupply`

`InSupply` means the corps can trace supply successfully. `Foraging`
means the corps cannot trace supply but can live off the area it occupies.
`OutOfSupply` means neither condition applies. All code in
`crates/core/src/supply.rs` uses these three states directly per
`PROMPT.md` §16.5.

## 2. Supply trace

Per `PROMPT.md` §16.5, a corps is `InSupply` when there is a continuous
path from the corps's current area to its capital or to a friendly depot.
For Phase 5 the implemented trace is:

1. Start in the corps's current area (`PROMPT.md` §16.5).
2. Traverse only friendly or neutral areas, i.e. areas that are not enemy-
   owned for the tracing power (`PROMPT.md` §16.5).
3. The path may not pass through enemy zones of control (`PROMPT.md` §16.5).
4. Enemy ZoC for Phase 5 is simplified to adjacency: any area adjacent to an
   enemy corps is in enemy ZoC (`PROMPT.md` §16.5).
5. If the trace reaches the owner's capital, the corps is `InSupply`
   (`PROMPT.md` §16.5).
6. If depot tracking exists in scenario state, reaching a depot also counts;
   Phase 5 code is written so capital tracing works even before depot state is
   added, exactly as allowed by `PROMPT.md` §16.5.

The trace is deterministic: traversal uses ordered collections only, matching
project-wide determinism rules and the §16.5 phase gate.

## 3. Foraging

Per `PROMPT.md` §16.5, a corps that fails the supply trace may still forage.
A corps is `Foraging` only when:

- it is not `InSupply`, and
- the area it occupies has `money_yield > 0`.

If `money_yield` is a placeholder, that does not authorize foraging; no value
is invented, per `PROMPT.md` §16.5 together with the global placeholder rule.

## 4. Attrition

Per `PROMPT.md` §16.5, corps that are unsupplied and not foraging lose SP each
turn from the attrition table. Phase 5 therefore applies attrition only to
`OutOfSupply` corps.

Attrition values come from `AttritionTable` rows and may remain
`Maybe::Placeholder`; when a row is a placeholder, Phase 5 applies no guessed
loss, because `PROMPT.md` §16.5 and §0 forbid invented numbers.

## 5. Depot orders

Per `PROMPT.md` §16.5, depot establishment is part of the supply subsystem.
Phase 5 validates that a depot order names a real area, belongs to a power with
live state, uses an owned or friendly area, and does not exceed the power's
`max_depots` limit where that limit exists in scenario data.

## 6. Events

Per `PROMPT.md` §16.5, supply resolution needs explicit event output. Phase 5
emits:

- `SupplyTraced { corps, supply_state }`
- `AttritionApplied { corps, sp_loss, reason }`

These events are the deterministic public record of the supply phase.
