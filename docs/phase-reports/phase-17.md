# Phase 17 — Modding

Date completed: 2026-04-25
Branch: `phase17-modding`
Gate: `docs/PROMPT.md` §16.17

## Summary

Implemented deterministic, data-driven mod loading for authored files.
This phase adds a `ModLoader`, mod manifest parsing, override
resolution, example mod content, and 15+ hand-written filesystem tests.
Mods may override scenario files, table files, and locale strings.
They may not override Rust code.

Workspace status after implementation: fmt, clippy, and full workspace
tests required before sign-off.

## Gate evidence

| §16.17 requirement | Status |
|---|---|
| Document mod structure and manifest | ✅ `docs/rules/modding.md` |
| Deterministic load order | ✅ `crates/core/src/mod_loader.rs` |
| Base first, mods alphabetical, later wins | ✅ Implemented and tested |
| Override resolution for authored data files | ✅ `resolve_file` + `load_with_mods` |
| Example mod provided | ✅ `mods/example_mod/` |
| 15+ hand-written tests | ✅ 16 tests |
| No code modding | ✅ Documented; loader handles data files only |

## What was built

### Documentation

- `docs/rules/modding.md`
  - mod directory structure under `mods/<mod_id>/`
  - `mod.json` manifest format
  - allowed override targets: scenarios, tables, locales
  - forbidden target: Rust code
  - deterministic load order and last-wins resolution
  - validation expectations matching base schema rules

### Data / examples

- `mods/.gitkeep`
- `mods/example_mod/mod.json`
- `mods/example_mod/tables/economy.json`
  - full economy-table structure using placeholder markers rather than
    invented authored values

### Core code

- `crates/core/src/mod_loader.rs`
  - `ModManifest`
  - `ModLoader::new`
  - `ModLoader::discover_mods`
  - `ModLoader::resolve_file`
  - `ModLoader::load_with_mods`
- `crates/core/src/lib.rs`
  - exports `pub mod mod_loader;`

## Test coverage

`crates/core/src/mod_loader.rs` includes 16 tests covering:

- empty mod directory discovery
- example mod discovery
- alphabetical sorting
- base-file fallback
- single-mod override
- last-mod-wins override
- typed loading through `load_with_mods`
- manifest serialization/deserialization
- nested paths
- no-active-mod fallback
- active mod without declared override
- missing manifest directory ignored
- missing active manifest fallback
- invalid JSON error path
- absolute-path passthrough

## Hard-rules compliance

- ✅ No floats
- ✅ No `HashMap` in simulation logic
- ✅ No invented rules-table numbers in example economy data
- ✅ 15+ tests shipped
- ✅ No performance benchmarks added in this phase

## Gate blocker note

**Q8 remains open.** Per `docs/questions.md`, reference-hardware access is
still missing for the performance benchmark gate, so this phase report
records that blocker instead of adding criterion/perf benchmark work.
