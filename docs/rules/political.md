# Political rules — canonical reference

Sourced from PROMPT.md §16.7 and the data shapes defined in
`gc1805_core_schema::tables::PpModifiersTable`.

## 1. State

Prestige points (PP) are tracked per power in
`Scenario.power_state[power].prestige` (i32).  Initial values come from
`PowerSetup.starting_pp`.

The `PpModifiersTable` (`data/tables/pp_modifiers.json`) maps event
names to integer PP deltas.  All entries are `Maybe::Placeholder` until
the designer provides concrete values (PROMPT.md §6.1).

## 2. PP gains and losses

PP deltas are applied via `apply_pp_delta(scenario, power, delta,
reason, tables)`.  If `tables.events[reason]` contains a
`Maybe::Value(v)`, the table value `v` is used instead of the passed
`delta`.  If the key is absent or `Maybe::Placeholder`, the passed
`delta` is used as-is.

All PP values in the tables file remain `Maybe::Placeholder` where
the designer has not specified concrete numbers.

## 3. Victory condition

The power with the highest PP at the scenario's `end` date wins.
Ties are resolved by the designer's tiebreaker rule (not yet
specified — flagged as a structural placeholder).

## 4. Revolt triggers

During the political phase, every area is scanned in BTreeMap order.
An area triggers a `RevoltTriggered` event if **all** of the following
hold:

1. `area.manpower_yield` is `Maybe::Value(y)` with `y > 0`.
2. `area.owner` is `Owner::Power(slot)`.
3. The owning power's `prestige < REVOLT_PRESTIGE_THRESHOLD`.

`REVOLT_PRESTIGE_THRESHOLD` is currently `0` — a structural placeholder.
The designer must provide the real threshold.

## 5. Peace conference scoring

A `PeaceConferenceOpened` event is emitted with the list of involved
powers.  The scoring rules reference PP totals to determine bargaining
strength.  The concrete mechanics are a structural placeholder pending
designer input.

## 6. Abdication condition

During the political phase, every power is scanned in BTreeMap order.
If a power's `prestige < ABDICATION_PRESTIGE_THRESHOLD` (currently
`-50`), an `AbdicationForced` event is emitted.

`ABDICATION_PRESTIGE_THRESHOLD` is a structural placeholder.  The
designer must provide the real value.

## 7. Phase resolution order

`resolve_political_phase` runs in this order:

1. **Check revolts** — iterate areas, emit `RevoltTriggered` events.
2. **Check abdication** — iterate powers, emit `AbdicationForced` events.

All iteration uses BTreeMap order for determinism (PROMPT.md §2.2).

## 8. Forbidden

- No floats anywhere.
- PP modifier values are designer-authored.  Phase 7 ships them as
  `Maybe::Placeholder`.
- Thresholds for revolt and abdication are structural placeholders,
  not designer-final numbers.

## 9. Test coverage

`crates/core/src/political.rs::tests` ships with 30 hand-written
cases covering:

- `apply_pp_delta`: positive, negative, zero, cumulative, table
  override, table placeholder, missing key, unknown power, large values.
- `check_revolts`: positive prestige, negative prestige, zero manpower,
  placeholder manpower, multiple areas, unowned area, minor-owned area,
  deterministic order.
- `check_abdication`: above threshold, below threshold, boundary,
  multiple powers, selective among three.
- `resolve_political_phase`: clean scenario, combined revolt+abdication,
  event ordering, determinism, multi-power selective.
