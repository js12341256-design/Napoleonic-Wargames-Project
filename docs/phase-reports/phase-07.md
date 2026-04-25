# Phase 7 — Political

Date completed: 2026-04-25
Branch: `phase7-political`
Gate: `docs/PROMPT.md` §16.7

## Summary

Political phase resolver covering: prestige-point tracking per power,
PP gains/losses via `PpModifiersTable` (all entries Placeholder until
designer provides values), revolt triggers for areas with positive
manpower yield owned by low-prestige powers, abdication condition for
extremely low prestige, and the `resolve_political_phase` entry point.
30 hand-written test cases.

All threshold values (`REVOLT_PRESTIGE_THRESHOLD = 0`,
`ABDICATION_PRESTIGE_THRESHOLD = -50`) are **structural placeholders**,
not designer-final numbers.  They exist so the code compiles, tests
run, and the shape of the logic is verifiable.  The designer must
provide real values before this gate can close.

## Gate evidence

| §16.7 requirement | Status |
|---|---|
| PP tracking in `power_state.prestige` | Done |
| PP gains/losses from `PpModifiersTable` | Done (all Placeholder) |
| Victory condition: highest PP at scenario end | Documented in `political.md` |
| Revolt triggers: area with manpower_yield>0, owner PP<threshold | Done, tests 10-16 |
| Peace conference scoring | Event variant added; mechanics are structural placeholder |
| Abdication condition | Done, tests 17-20 |
| Determinism (BTreeMap iteration) | Tests 15, 24 |
| 20+ hand-written test cases | 30 tests |

## What was built

### `gc1805-core-schema`

- `events.rs` — Four new `Event` variants:
  - `PrestigeAwarded { power, delta, reason }`
  - `RevoltTriggered { area, owner }`
  - `PeaceConferenceOpened { powers }`
  - `AbdicationForced { power }`

### `gc1805-core`

- `political.rs` — New module with:
  - `apply_pp_delta(scenario, power, delta, reason, tables) -> Event`
  - `check_revolts(scenario) -> Vec<Event>`
  - `check_abdication(scenario) -> Vec<Event>`
  - `resolve_political_phase(scenario, tables) -> Vec<Event>`
  - 30 unit tests
- `lib.rs` — `pub mod political;` added

### `docs/rules/`

- `political.md` — Full rules reference for the political phase

## Structural placeholders (not designer-final)

- `REVOLT_PRESTIGE_THRESHOLD = 0` — revolt fires when power prestige < 0
- `ABDICATION_PRESTIGE_THRESHOLD = -50` — abdication fires when prestige < -50
- All `PpModifiersTable.events` entries are `Maybe::Placeholder`
- Peace conference scoring mechanics are not yet specified
- Victory tiebreaker rule is not yet specified

## Hard rules compliance

- No floats anywhere
- No invented numerical values (all PP table entries are Placeholder)
- 30 hand-written test cases (exceeds 20+ requirement)
- BTreeMap only in simulation logic (no HashMap)
- Other phases not modified
