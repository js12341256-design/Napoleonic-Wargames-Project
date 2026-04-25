# Land Combat rules — canonical reference

Sourced from PROMPT.md §16.4 (land-combat resolution), §6.1 (placeholders),
and the data shapes defined in `gc1805_core_schema::tables::CombatTable` and
`::MoraleTable`.

## 1. Triggering a battle

A `ATTACK` order (see `crates/core/src/orders.rs`) names one or more
attacking corps, a target area, and a formation string.  Before resolution,
the order passes `validate_attack` (PROMPT.md §16.4):

1. The attacking corps list must be non-empty.
2. Every listed corps must exist in `scenario.corps`.
3. All listed corps must be owned by `order.submitter`.
4. `order.target_area` must exist in `scenario.areas`.
5. At least one attacking corps must occupy an area adjacent to
   `target_area` (check `scenario.adjacency`).
6. `target_area` must contain at least one corps not owned by
   `order.submitter` (an enemy must be present).
7. `order.submitter` must be at `DiplomaticState::War` with the owner
   of at least one defending corps.
8. `order.formation` must be a non-empty string.  Table-level existence
   of the formation key is checked at resolve time, not validation time,
   so an unknown formation resolves with zero column shift.

## 2. Force ratio → bucket (PROMPT.md §16.4)

Attacker SP = sum of `infantry_sp + cavalry_sp + artillery_sp` for all
corps in `order.attacking_corps`.

Defender SP = sum over all corps in `target_area` not owned by `submitter`.

If defender SP is 0 the order is rejected with `reason_code = "NO_DEFENDER"`.

The ratio bucket (the CRT column) is selected by integer arithmetic only —
**no floats** (PROMPT.md §2.2):

| Condition            | Bucket |
|----------------------|--------|
| `att >= 3 * def`     | `3:1`  |
| `att >= 2 * def`     | `2:1`  |
| `att * 2 >= 3 * def` | `3:2`  |
| `att >= def`         | `1:1`  |
| `att * 2 >= def`     | `1:2`  |
| otherwise            | `1:3`  |

## 3. Column shifts (PROMPT.md §16.4)

Column shifts are applied before looking up the die result.  The net shift
is: `att_col_shift - def_col_shift` applied to the base die index (clamped
to `[0, die_faces - 1]`).

**Formation shifts** — look up the key
`"<ATTACKER_FORMATION>_vs_<DEFENDER_FORMATION>"` in
`CombatTable.formation_matrix`.  The `att_col_shift` and `def_col_shift`
come from the matching `FormationEntry`, or 0 if the key is absent.

**Terrain shifts** — look up the terrain of `target_area` (e.g. `"MOUNTAIN"`)
in `CombatTable.terrain_modifiers`.  The `att_col_shift` value is added to
the attacker's column shift.  Terrain does not shift the defender column.

## 4. Die roll → result (PROMPT.md §16.4)

```
die_index  = rng_seed % die_faces
adj_index  = (die_index as i32 + att_col_shift - def_col_shift)
               .clamp(0, die_faces - 1)
result     = CombatTable.results[bucket][adj_index]
```

If `result` is `Maybe::Placeholder`, the resolver returns an
`OrderRejected { reason_code: "COMBAT_TABLE_PLACEHOLDER", … }` event.
**Gate cannot close until the human designer fills in real values (Q1
in docs/questions.md).**

## 5. SP loss application (PROMPT.md §16.4)

`CombatResult.attacker_sp_loss` SP are removed from attacking corps.
`CombatResult.defender_sp_loss` SP are removed from defending corps.

Distribution: divide the total loss evenly across corps.  Assign the
quotient to each corps.  The remainder (`loss % corps_count`) is assigned
to the first corps (lowest CorpsId, lex order).

A corps reduced to 0 SP or below stays in `scenario.corps` with its SP
clamped to 0.  Removal is Phase 10's job (PROMPT.md §16.4).

## 6. Morale delta (PROMPT.md §16.4)

`CombatResult.attacker_morale_q4` is added to the `morale_q4` of every
attacking corps.  `CombatResult.defender_morale_q4` is added to every
defending corps.  Both are negative in typical losing-side rows.

> **Q4 (designer):** What are the concrete morale delta values per row?
> See `docs/questions.md` Q4.  Until answered, all morale fields in the
> combat table are `{"_placeholder": true}`.

## 7. Outcome determination (PROMPT.md §16.4)

After morale deltas are applied, compare post-battle morale of each side
against `MoraleTable.rout_threshold_q4` and `MoraleTable.retreat_threshold_q4`.
If either threshold is `Maybe::Placeholder`, use `AttackerRepulsed` as the
safe default (no movement committed).

| Priority | Condition                                      | Outcome              |
|----------|------------------------------------------------|----------------------|
| 1        | defender morale < rout_threshold               | `DefenderRouted`     |
| 2        | defender morale < retreat_threshold            | `DefenderRetreats`   |
| 3        | attacker morale < retreat_threshold            | `AttackerRepulsed`   |
| 4        | (neither side forced back)                     | `MutualWithdrawal`   |

## 8. Retreat resolution (PROMPT.md §16.4)

For `DefenderRetreats`:

1. Candidate areas: all areas adjacent to `target_area` (from
   `scenario.adjacency`) that are **not** in the attacker's ZoC.
2. Tiebreak: lexicographic order on `AreaId`.
3. If no candidate exists, the corps stays in place (surrounded).
4. The corps' `area` field is updated.  Emit `Event::CorpsRetreated`.

For `DefenderRouted`:

- Each defending corps emits `Event::CorpsRouted`.  Routed corps remain
  in their area; the game engine may apply further morale penalties in
  Phase 10.

## 9. Leader casualty check (PROMPT.md §16.4)

After battle, if a corps with a leader present had SP losses,
a `LeaderCasualty` check is performed using `LeaderCasualtyTable.by_intensity`.
The intensity bucket is designer-authored.  Until filled, results are
`Maybe::Placeholder` and no event is emitted.

## 10. Zones of Control (PROMPT.md §16.4)

A power's Zone of Control (`ZoC`) is the set of land areas adjacent to
any area in which that power has at least one corps, **excluding** areas
occupied by the power's own corps.

ZoC affects movement: an enemy corps may not exit an area in the
opponent's ZoC unless there are no legal non-ZoC exits (Phase 2 movement
rules wire this check; combat resolver uses ZoC for retreat candidate
filtering only).

`zones_of_control(scenario, power)` is a pure function that returns
`BTreeSet<AreaId>`.
