# Naval Rules

Phase 9 implements the naval skeleton described by `docs/PROMPT.md` §16.9.

## Sea zones as a graph

Sea movement uses the scenario's `sea_zones` plus `sea_adjacency` to build a deterministic undirected graph. Pathfinding is hop-based and uses ordered containers so identical inputs produce identical results.

## Port access via `CoastLink`

Ports connect land and sea through `Scenario.coast_links`. A corps may only embark or disembark through a land area marked `port: true` that also has a matching `CoastLink` to the fleet's current sea zone.

## Fleet movement

Fleet movement is hop-based across connected sea zones. Validation requires:

- the fleet exists,
- the fleet is owned by the submitting power,
- the destination sea zone exists,
- and a sea path exists in the graph.

Successful movement updates the fleet's `at_sea` position and emits `FleetMoved`.

## Naval combat

Naval combat resolves by integer ratio bucket using `NavalCombatTable`. Phase 9 does **not** invent combat values. The shipped table is still placeholder-only, so placeholder results reject resolution with `NAVAL_TABLE_PLACEHOLDER` until designers author real values.

## Blockade

A fleet in a sea zone adjacent to an enemy port establishes a blockade through that port connection. Phase 9 emits `BlockadeEstablished` and marks the enemy port area as blockaded.

## Transport

Transport is modeled as embark/disembark through ports:

- `CorpsEmbarked` when a corps boards a fleet from a linked port.
- Fleet movement while embarked corps remain attached to the fleet.
- `CorpsDisembarked` when the corps lands at a linked destination port.

## Weather

`WeatherTable` is introduced for naval/weather integration, but the authored file remains placeholder/minimal in Phase 9. No weather effects are invented here.
