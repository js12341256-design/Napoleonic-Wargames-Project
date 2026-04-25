# Diplomacy rules — canonical reference

Sourced from PROMPT.md §16.6 for every rule in this file.

## 1. Diplomatic state model (§16.6)

Each major-power pair is in exactly one symmetric diplomatic state,
keyed canonically by `DiplomaticPairKey(lo, hi)` in the scenario’s
`BTreeMap` diplomacy store (§16.6):

- `ALLIED` (§16.6)
- `FRIENDLY` (§16.6)
- `NEUTRAL` (§16.6)
- `UNFRIENDLY` (§16.6)
- `WAR` (§16.6)

If a pair is absent from `Scenario.diplomacy`, Phase 6 treats it as
`NEUTRAL` (§16.6).

## 2. Diplomatic order vocabulary (§16.6)

Phase 6 recognizes these diplomatic actions (§16.6):

- `DeclareWar { submitter, target }` (§16.6)
- `ProposePeace { submitter, target, terms }` (§16.6)
- `FormAlliance { submitter, target }` (§16.6)
- `BreakAlliance { submitter, target }` (§16.6)
- `SendSubsidy` / subsidy proposal (§16.6)

`SendSubsidy` is split across phases (§16.6): the diplomatic intent is
part of the Phase 6 order vocabulary, while the actual money transfer is
resolved later by the Phase 3 economic machinery.

## 3. Validation rules (§16.6)

Validation is pure and never mutates the scenario (§16.6):

- `DeclareWar` is legal only if both powers exist, the target differs
  from the submitter, and the pair is not already at `WAR` (§16.6).
- `ProposePeace` is legal only if the two powers are currently at `WAR`
  (§16.6).
- `FormAlliance` is legal only if the pair is not already `ALLIED` and
  is not currently at `WAR` (§16.6).
- `BreakAlliance` is legal only if the pair is currently `ALLIED`
  (§16.6).

## 4. Resolution order (§16.6)

Diplomatic orders are secret while pending and are revealed only during
resolution (§16.6). In implementation terms, the submitted order list is
the pending queue for the phase, and the reveal surface is the emitted
`Event` stream at resolution time (§16.6).

Phase 6 resolves in a fixed nine-step order (§16.6):

1. Declare war orders (§16.6)
2. Break alliance orders (§16.6)
3. Form alliance orders (§16.6)
4. Propose peace orders (§16.6)
5. Reserved (§16.6)
6. Reserved (§16.6)
7. Reserved (§16.6)
8. Reserved (§16.6)
9. Reserved (§16.6)

Within each implemented step, orders are processed in deterministic
`BTreeMap` submitter order (§16.6).

## 5. Declare war resolution (§16.6)

When `DeclareWar` resolves (§16.6):

1. The pair state becomes `WAR` (§16.6).
2. `WarDeclared { by, against }` is emitted (§16.6).
3. Prestige / PP change for the acting power is read from
   `PpModifiersTable.events["declare_war"]` (§16.6).
4. If that table entry is `Maybe::Value(delta)`, prestige changes by the
   authored integer amount and `PrestigeChanged { power, delta, reason }`
   is emitted (§16.6).
5. If that table entry is `Maybe::Placeholder`, no prestige delta is
   invented and no prestige-change event is emitted (§16.6 + PROMPT.md §0).

## 6. Alliance cascade (§16.6)

Alliance obligations are checked immediately after a war declaration
(§16.6).

If power `A` is allied to `B`, and `B` is at war with `C`, then `A`
enters war with `C` unless an exception applies (§16.6). In Phase 6,
the implemented exception is: if the prospective new belligerent is
already at `WAR` with that attacker, no new state change or cascade event
is emitted (§16.6).

Each newly pulled-in power emits:

- `AllianceCascade { new_belligerent, against, via_ally }` (§16.6)

The cascade walks the alliance network deterministically, so chained
alliances resolve in stable order (§16.6).

## 7. Alliance and peace resolution (§16.6)

- `BreakAlliance` sets the pair state to `NEUTRAL` and emits
  `AllianceBroken { power_a, power_b }` (§16.6).
- `FormAlliance` sets the pair state to `ALLIED` and emits
  `AllianceFormed { power_a, power_b }` (§16.6).
- `ProposePeace` does not immediately end the war in Phase 6; it emits
  `PeaceProposed { by, to }` and leaves acceptance to a later phase
  (`PeaceAccepted` is reserved for that follow-up) (§16.6).

## 8. PP / prestige hooks (§16.6)

Diplomatic PP costs and gains are data-authored in `PpModifiersTable`
(§16.6). Per PROMPT.md §0 and §6.1, Phase 6 never invents numeric values.
Any missing or not-yet-authored diplomatic modifier remains
`Maybe::Placeholder` (§16.6).

The Phase 6 implementation wires `declare_war` directly today (§16.6).
Additional diplomatic modifiers remain schema-level hooks until the
prompt defines their exact application semantics (§16.6).
