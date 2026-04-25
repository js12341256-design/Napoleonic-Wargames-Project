# Modding rules — canonical reference

Sourced from `docs/PROMPT.md` §16.17. This phase adds file-based mod
loading only. Mods may replace authored data files, but they do not
change Rust code or the deterministic simulation rules engine itself.

## 1. Mod directory structure

Mods live under `mods/<mod_id>/`.

Minimum structure:

```text
mods/
  <mod_id>/
    mod.json
    scenarios/
    tables/
    locales/
```

Only files inside a mod directory are considered part of that mod.
The `mod_id` should be stable, lowercase, and match the manifest's
`id` field.

## 2. Manifest format

Each mod must provide `mods/<mod_id>/mod.json` with this shape:

```json
{
  "id": "example_mod",
  "name": "Example Mod",
  "version": "0.1.0",
  "description": "A minimal example mod showing override structure.",
  "overrides": ["tables/economy.json"]
}
```

Fields:

- `id` — stable mod identifier.
- `name` — display name.
- `version` — mod version string.
- `description` — human-readable summary.
- `overrides` — relative file paths that this mod intentionally
  overrides.

`overrides` entries are always relative to the base data root. Example:
`tables/economy.json` means the mod may provide
`mods/<mod_id>/tables/economy.json`.

## 3. What mods may override

Allowed override targets in this phase:

- scenario files (`scenarios/...`)
- rules table files (`tables/...`)
- locale strings (`locales/...`)

Not allowed:

- Rust source code
- Cargo manifests
- CI configuration
- any executable logic outside authored data files

This keeps mods data-driven, matching the PROMPT rule that authored
values live in files and the simulation stays deterministic.

## 4. Load order

Load order is deterministic:

1. Base game content is loaded first.
2. Mods are discovered from `mods/`.
3. Mods are sorted alphabetically by mod ID.
4. Later entries override earlier entries.

When resolving one file, the loader checks active mods in reverse load
order so the last active mod wins. If no active mod overrides the file,
the base game file is used.

## 5. Validation

A modded file must pass the same schema and deserialization rules as the
base game file it replaces.

That means:

- scenario overrides must deserialize into the same scenario schema
- table overrides must deserialize into the same table schema
- locale overrides must preserve the expected data format

A mod cannot bypass validation by replacing a file. Invalid JSON,
invalid schema shape, or missing required fields are load errors.

## 6. Determinism and scope

Phase 17 modding is intentionally narrow:

- no code plugins
- no scripting runtime
- no dynamic rule execution
- no performance benchmark work in this phase

Per the current project blockers, the performance-benchmark gate remains
blocked by Q8 and is tracked in the phase report rather than implemented
here.
