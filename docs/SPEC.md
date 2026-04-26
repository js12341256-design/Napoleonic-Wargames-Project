# SPEC.md — placeholder

**Status: placeholder. Do not treat as authoritative.**

The master prompt (`PROMPT.md`) references a separate design spec in §5 (data
model), §11.2 (diplomatic actions), and §21 (screen list). That document was
not included with the prompt.

Per ADR 0001, until a designer provides a separate spec, PROMPT.md itself is
treated as the complete specification. Gaps surfaced during implementation
are logged in `questions.md` and, once resolved, in `adjudications.md`.

## Sections the prompt assumed existed here

- Full typed data model for powers, leaders, corps, fleets, areas, sea zones,
  and diplomatic state. (§5)
- Complete list of diplomatic actions with their validator rules. (§6.7,
  §11.2)
- Enumeration of every screen in the desktop and web clients. (§16.15
  requires "all screens in Section 21 of the spec".)
- Scenario-level balance notes (e.g. starting force levels, PP values).

## When to replace this file

Replace when the designer delivers a real spec. At that point:

1. Overwrite this file with the spec content.
2. Walk `questions.md` for entries that can now be answered.
3. Bump `scenario.rules_version` if any table semantics change.
