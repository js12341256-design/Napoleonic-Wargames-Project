# Minor countries

Phase 8 introduces deterministic minor-country handling for the 1805
scenario.

## Scenario representation

The authored scenario keeps a compact `MinorSetup` per minor:

- `display_name`
- `home_areas`
- `initial_relationship`
- `patron`
- `starting_force_level`

The richer Q6 payload is preserved in `data/tables/minors.json` so later
phases can consume:

- primary / secondary areas
- capital area
- force-pool caps
- yields
- fortresses
- diplomatic inclinations
- special rules
- formed-minor eligibility

## Runtime state machine

`crates/core/src/minors.rs` exposes the runtime-facing state machine:

- `Independent`
- `AlliedFree { patron }`
- `Feudal { patron }`
- `Conquered { by }`
- `InRevolt`

This is intentionally a thin layer over the authored `MinorSetup` fields,
so Phase 8 can ship without reshaping the persisted scenario root.

## Activation

`activate_minor(scenario, minor_id, tables, rng_seed)` is deterministic:

- If the activation row is present, outcomes are selected from the row's
  ordered weighted distribution.
- If the row is absent or placeholder, the fallback uses `(rng_seed % 6)`.
- The scenario's stored minor relationship is updated in place.
- A `MinorActivated` event is emitted.

Fallback mapping for placeholder activation rows:

- 0–1 → `Independent`
- 2 → `AlliedFree`
- 3 → `Feudal`
- 4 → `Conquered`
- 5 → `InRevolt`

The patron chosen for placeholder activation is deterministic: current
patron if present, otherwise the lexicographically first power in the
scenario, otherwise `FRA` as a final no-powers-present fallback.

## Control validation

`validate_minor_control(scenario, power, minor)` checks whether a major has
recognized control of a minor:

- `Independent` and `InRevolt` always fail.
- `AlliedFree` / `Feudal` require diplomatic patronage by the caller.
- `Conquered` requires the caller to be the conqueror.

## Data notes

The Q6 source file declares and the generated scenario now includes the full
minor roster for the 1805 setup. Placeholder areas were added anywhere the
scenario lacked an area referenced by a minor's home territory, capital, or
fortress.

These placeholder areas intentionally keep:

- placeholder money yield
- placeholder manpower yield
- deterministic zeroed map coordinates
- terrain inferred from the minor kind

They exist to satisfy Phase 8 structure and validation, not to claim final
map authoring quality.
