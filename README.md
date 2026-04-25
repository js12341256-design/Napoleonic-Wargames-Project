# Grand Campaign 1805

An original, clean-room Napoleonic grand-strategy game. Seven great powers, point-to-point map, monthly turns from April 1805 through December 1815.

**Current status:** all 19 roadmap phases are now represented structurally across the integration branch, later phase branches, and this Phase 19 polish pass. That does **not** mean the game is publicly playable yet: placeholder data and unresolved human-design questions still block release.

See also:

- `docs/RELEASE_CHECKLIST.md`
- `docs/ARCHITECTURE.md`
- `docs/questions.md`
- `docs/phase-reports/`

## Repository map

| Path | Purpose |
|---|---|
| `crates/core` | Pure deterministic simulation library |
| `crates/core-schema` | Typed data schemas, canonical JSON, IDs, events |
| `crates/core-rng` | Seeded deterministic RNG utilities |
| `crates/core-validate` | Order validators used by CLI/server/clients |
| `crates/ai` | AI decision layer |
| `crates/server` | Authoritative multiplayer server |
| `crates/netcode` | Protocol/message types |
| `crates/client-shared` | Logic shared between desktop and web clients |
| `crates/client-desktop` | Desktop client shell (Bevy) |
| `crates/client-web` | Web/WASM bridge |
| `crates/cli` | Headless runner and smoke-test surface |
| `tools/xtask` | Workspace automation |
| `tools/asset-pipeline` | Offline asset/content processing |
| `data/` | Scenario JSON, rules tables, map data |
| `docs/` | Prompt, ADRs, rules docs, reports, release docs |
| `reference/prototype/` | Visual reference only |

## Phase status

Structurally, the roadmap has been pushed through Phase 19. The repository evidence for this is split between the current integration branch and dedicated phase branches.

Open gates still called out by the project docs:

- **Phase 4:** real combat tables still need human-authored values (Q1)
- **Phase 8:** full minor-country list still missing (Q6)
- **Phase 16:** human translators still required (Q7)

Additional public-release blockers:

- license decision (Q4)
- target hardware/perf sign-off (Q8)
- real Rust toolchain pin instead of floating `stable` (Q9)
- WASM target verification and full CI matrix verification

## Build

```sh
cargo build --workspace
```

## Test

```sh
cargo test --workspace
```

## Lint

```sh
cargo clippy --workspace --all-targets -- -D warnings
```

## CLI usage

Load the standard 1805 scenario:

```sh
cargo run -p gc1805-cli -- load data/scenarios/1805_standard/scenario.json
```

Run the smoke test added in Phase 19:

```sh
cargo run -p gc1805-cli -- smoke-test
```

Other useful commands already present on this branch:

```sh
cargo run -p gc1805-cli -- move-all-to-capital data/scenarios/1805_standard/scenario.json
cargo run -p gc1805-cli -- economic-phase data/scenarios/1805_standard/scenario.json
```

## Open blockers before playable release

This repository is still **closed-beta / internal-test** material, not a public playable release.

Do not treat the 1805 scenario as release-ready until all of the following are resolved:

- placeholder tables replaced with human-authored rules values
- combat table sign-off complete
- full minor-country list delivered
- translation pass complete for required locales
- license committed
- reference hardware benchmark sign-off complete
- real Rust version pin committed
- WASM and cross-platform CI matrix verified on the integration branch

## Branch structure

### Main integration branch

- `claude/implement-design-system-ciX1R` — current integration branch carrying Phases 0–4 and serving as the base for this Phase 19 polish branch.

### Phase branches present on origin

- `phase5-supply`
- `phase6-diplomacy`
- `phase7-political`
- `phase9-naval`
- `phase10-turn-loop`
- `phase12-server`
- `phase13-pbem`
- `phase14-desktop-ui`
- `phase15-web-ui`
- `phase16-localization`
- `phase17-modding`
- `phase18-replay`

### Current polish branch

- `phase19-polish`

### Notes

- `main` is not the active integration branch for current subsystem work.
- Branch presence is not the same thing as public-release readiness.
- The release checklist is the authority for what is still open.

## Ground rules

- Read `docs/PROMPT.md` §0 before changing code.
- Do not invent rules values.
- Keep simulation logic deterministic.
- Treat `Maybe<T>` placeholders and `unplayable_in_release: true` as real release gates, not as TODO decoration.

## License

Currently unlicensed / all rights reserved pending the explicit decision tracked in `docs/questions.md` Q4.
