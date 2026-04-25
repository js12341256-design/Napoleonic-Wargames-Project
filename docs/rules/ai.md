# AI rules

Phase 11 implements a deterministic, projection-only AI layer aligned with `docs/PROMPT.md` §15.

## 1. Four-layer AI

1. **Projection input only** — AI reads the projected `Scenario` passed into `AiContext`; it must not inspect hidden state.
2. **Personality layer** — `AiPersonality` supplies stable integer preferences for aggression, defensiveness, diplomatic openness, and economic priority.
3. **Heuristic generation layer** — movement, economic, and diplomatic orders are produced with pure deterministic rules.
4. **Order emission layer** — AI emits typed `Order` values only; it does not mutate scenario state directly.

## 2. Personality configuration

Default personality data lives at `data/ai/personalities/default.json`.
All personality values are integers. No floats are permitted.
Missing fields fall back to the canonical neutral defaults of `5`.

## 3. Deterministic seeding

Tie-breaking uses a stable string hash plus the caller-supplied `rng_seed`.
Given the same projected scenario, power, personality, and seed, AI output must be identical across runs and platforms.

## 4. Projection-only discipline

Per `docs/PROMPT.md` §15, the AI is non-cheating. It operates only on the projected scenario provided by the caller. Hidden enemy units, unseen queues, and any non-projected information are out of bounds.

## 5. No invented values

Per `docs/PROMPT.md` §15 and the project operating principles, AI does not invent rule numerics or synthetic unit data.
When it builds a corps order, it derives composition from existing visible corps data instead of fabricating a new template.
If the required data is absent, it emits no order.
