# Rules adjudications

Every ambiguous rule encountered during implementation, the interpretation
chosen, and the reasoning. Format per PROMPT.md §21.2.

---

## Adjudication 0001 — Interception scope at Phase 2

Date: 2026-04-25
Rule reference: `docs/rules/movement.md` §4
Phase: 2 (movement)

### Ambiguity

PROMPT.md §16.3 requires "forced march, interception, and stacking
rules implemented" at the Phase 2 gate.  Interception, however, depends
on three subsystems that come later:

1. **Supply trace** (Phase 5).  Interception of an unsupplied force is
   resolved differently than a supplied one.
2. **Diplomatic state** (Phase 6).  An interception against an allied
   power is illegal.
3. **Impulse queue** (Phase 10).  Interception triggers between
   impulses, so without an impulse model the resolver cannot fire.

A faithful implementation of interception at Phase 2 is therefore not
possible without inventing semantics that later phases would need to
overwrite.  Per PROMPT.md §0 ("correctness over cleverness"; "do not
guess") that is forbidden.

### Chosen interpretation

`Order::Interception { corps, target_area, conditions }` is typed and
syntactically validated at Phase 2:

- the corps exists,
- the target area exists,
- the conditions parse against the conditional-order grammar (Phase 6),
- the corps is owned by the submitting power.

The resolver returns `MovementResolution::Pending` and the order
remains in the order book.  Phase 5 adds the supply check; Phase 6 the
diplomatic check; Phase 10 wires the actual firing.

### Rationale

This treats the Phase 2 gate item as "interception is *typed and
queueable*" rather than "interception is *resolvable*."  The visible
behaviour to a player at end of Phase 2 is that an interception order
is accepted but never fires — which is acceptable because no other
phase has reached the impulse model yet.  No invented numbers, no
silent fudge.

### Test case

`testdata/rules_cases/movement/interception_pending_01.yaml` — submit
an interception, assert `MovementResolution::Pending`.

### Closure criteria

Close this adjudication when Phase 10 lands.  At that point the
resolver gains a real fire condition and the test should be updated
to assert the post-impulse outcome.
