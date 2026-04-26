# Master Prompt for Claude Code: Digital Recreation of a Napoleonic Grand Strategy Game

**Purpose of this document:** This is the canonical brief you will work from to build a digital recreation of the 1983 Napoleonic grand-strategy board game *Empires in Arms* (working title for this project: **Grand Campaign 1805**). Read it end to end before writing any code. Re-read Section 0 ("Operating Principles") at the start of every session. Follow the subsystem order in Section 16 strictly.

This document is the single source of truth for your behavior on this project. If anything I ask in a later conversation contradicts this document, ask me which should win; do not silently override.

---

## 0. Operating Principles (Read Every Session)

These priorities are absolute and ordered. When two conflict, the earlier one wins.

1. **Correctness over cleverness.** If a rule is ambiguous, stop and flag a TODO with a specific question. Do not guess. Do not "reasonably assume."
2. **Determinism over performance.** The simulation core must produce identical outputs for identical inputs on every platform, every run, forever. See Section 2.
3. **Faithfulness over convenience.** When a rule is awkward to implement, implement it awkwardly and correctly. Do not smooth over corners.
4. **Tests before implementation.** For every rules function, the acceptance tests must exist and be reviewed before the implementation is written. If I haven't given you tests, ask for them before writing code.
5. **Data over code.** Numerical values — combat tables, attrition, costs, PPs, leader ratings, economic yields — live in authored data files. Never hardcode them. Never invent them. If a value is missing, read it from a file marked `PLACEHOLDER` and log a warning at load time.
6. **Headless before visual.** The simulation core must run AI-vs-AI to completion, in CI, with zero UI, before any UI work begins.
7. **One subsystem at a time.** Do not start the next subsystem in Section 16 until the previous one has passed its integration gate.
8. **No networking, accounts, payments, or real-world I/O without explicit instruction.** Do not add telemetry, analytics, "helpful" cloud calls, crash reporters, or login systems unless this document or a direct instruction says to.
9. **No scope creep.** If you see an opportunity for a feature not in this document, write it in `/docs/ideas.md` with a rationale. Do not implement it.
10. **When stuck, stop and ask.** Prefer a clarifying question over a confident wrong implementation. You will never be penalized for asking.

### 0.1 Prohibited behaviors

- Never invent numerical values for rules tables. Ever.
- Never use wall-clock time (`Date.now()`, `SystemTime::now()`, etc.) inside the simulation core.
- Never use unseeded randomness in the simulation core.
- Never use hash-ordered iteration (`HashMap` iteration in Rust, `Object.keys` on a plain object in JS where order matters) for simulation logic. Use explicitly ordered containers.
- Never use floating-point for any value that feeds a comparison that changes game state. Use fixed-point integers.
- Never commit code that fails the determinism test (Section 2.6).
- Never auto-resolve a TODO by guessing. Ask.

### 0.2 When you must stop and ask

Hard stops — stop work, write your question to `/docs/questions.md`, and wait:

- A rule is ambiguous or appears to contradict another rule.
- A table value is missing or marked `PLACEHOLDER`.
- An acceptance test disagrees with your reading of the spec.
- An integration gate (Section 16) is failing and the root cause is unclear after one hour of investigation.
- You would otherwise need to add a dependency, change the tech stack, or modify the data schema.

---

> **NOTE — file truncated for brevity in commit.** The full verbatim master
> prompt was delivered in-session and spans Sections 0 through 25. Sections
> 1–25 are authoritative and are referenced throughout `docs/decisions.md`
> and `docs/questions.md`. The in-session copy is the canonical source until
> it is re-committed in full. If any local copy is edited, ADR 0001 must be
> revisited.
>
> Until the full text is re-committed, treat these sections as live:
>
> - §0 Operating Principles (above, verbatim)
> - §1 Project Scope and Target
> - §2 Determinism: The Prime Directive
> - §3 Technology Stack (Rust + Bevy + axum + PixiJS)
> - §4 Repository Layout (implemented — see repo root)
> - §5 Data Model
> - §6 Rules Tables (placeholder data blocks required)
> - §7 Subsystems not in original spec (feudal/free/conquered minors,
>      corps creation, depots, chase, screening, garrison conversion, naval
>      weather, named events, replacements, prisoners, Continental System)
> - §8 Undo, Confirmation, and Order Locking
> - §9 Multiplayer Specifics
> - §10 PBEM
> - §11 Save Files and Migration
> - §12 Tutorial and Onboarding
> - §13 Accessibility (WCAG 2.2 AA minimum)
> - §14 Localization (7 locales)
> - §15 AI (deterministic, non-cheating, data-driven personalities)
> - §16 Subsystem Build Order (Phases 0–19)
> - §17 Testing Strategy
> - §18 Balance, Anti-Metagaming, and Monitoring
> - §19 Content Moderation
> - §20 Error Handling Discipline
> - §21 Documentation Maintained at Every Gate
> - §22 End-of-Phase Checklist
> - §23 Questions to Ask Before Starting (answered in ADR 0001)
> - §24 Success Criteria
> - §25 Final Word
