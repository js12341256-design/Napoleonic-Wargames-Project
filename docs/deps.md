# Dependency justifications

PROMPT.md §3.1: every new dependency requires justification here. Prefer
fewer, smaller, well-maintained crates.

---

## Workspace-wide

| Crate | Used in | Why |
|---|---|---|
| `serde` + `serde_json` | `core`, `core-schema` | Canonical JSON serialization per §5.2. Required by prompt. |
| `schemars` | `core-schema` | JSON schema generation per §16.2 gate. Required by prompt. |
| `blake3` | `core` | State hashing per §2.5. Required by prompt. |
| `rand` + `rand_chacha` | `core-rng` | Counter-based, deterministic RNG per §2.2. Required by prompt. |
| `indexmap` | `core` | Insertion-ordered maps for deterministic iteration per §2.2. Required by prompt. |
| `thiserror` | all | Typed error variants per §20.1. Required by prompt. |

## `server`

| Crate | Why |
|---|---|
| `tokio` | Async runtime per §3.2. |
| `axum` | HTTP + WebSocket server per §3.2. |
| `sqlx` | Compile-time-checked SQL per §3.2. |

## `client-desktop`

| Crate | Why |
|---|---|
| `bevy` | Engine per §3.3. |

## Prohibited in `core`

See PROMPT.md §3.1. Do not pull `std::time`, async runtimes, or any I/O
crate into the `core` crate.
