# Grand Campaign 1805

An original, clean-room Napoleonic grand-strategy game. Seven great powers,
point-to-point map, monthly turns from April 1805 through December 1815.

**Status: Phase 0 scaffolding complete.** No playable code yet.
See `docs/PROMPT.md` §16 for the build roadmap.

## Repository map

| Path                     | Purpose                                                |
|--------------------------|--------------------------------------------------------|
| `crates/core`            | Pure simulation library (deterministic, no I/O)        |
| `crates/core-schema`     | Typed data schemas                                     |
| `crates/core-rng`        | Seeded, counter-based RNG with named streams           |
| `crates/core-validate`   | Order validators                                       |
| `crates/ai`              | AI decision layer (operates on projections)            |
| `crates/server`          | Authoritative server (axum + tokio)                    |
| `crates/netcode`         | Client–server protocol types                           |
| `crates/client-shared`   | Logic shared between desktop and web clients           |
| `crates/client-desktop`  | Desktop client (Bevy)                                  |
| `crates/client-web`      | Web client WASM glue (TS/React shell under `/web`)     |
| `crates/cli`             | Headless runner                                        |
| `tools/xtask`            | Workspace automation                                   |
| `tools/asset-pipeline`   | Map / locale / sprite processor                        |
| `data/`                  | Scenario JSON, rules tables, map data (human-authored) |
| `docs/`                  | Spec, ADRs, open questions, rules references           |
| `reference/prototype/`   | *The Dusk of the Old World* — visual reference only    |

## Building

```sh
cargo build --workspace
cargo test --workspace
```

Node 22 (see `.nvmrc`) is used for the web client under `/web`.

## Ground rules

- Read `docs/PROMPT.md` §0 before making any change.
- No invented rules values. Missing table values stay `PLACEHOLDER` and the
  scenario is marked `unplayable_in_release: true`. See `docs/questions.md` Q1.
- Determinism is non-negotiable. See `docs/PROMPT.md` §2.
- Every rules change requires a `RULES:` commit and a regenerated determinism
  golden. See `docs/PROMPT.md` §2.6.

## License

Unlicensed (all rights reserved) pending `docs/questions.md` Q4.
