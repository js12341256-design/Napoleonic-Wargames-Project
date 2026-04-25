# Phase 16 — Localization

Date completed: 2026-04-25
Branch: `phase16-localization`
Gate: `docs/PROMPT.md` §16.16 / §14

## Summary

Implemented the localization scaffold for seven required locales:
English (`en`) as the canonical source, pseudo-locale (`zz`) with
bracket-wrapped strings for layout testing, and placeholder locale files
for French, German, Spanish, Russian, Polish, and Italian.

Added `gc1805-client-shared::locale` with a deterministic manual YAML
loader backed by `BTreeMap`, key-fallback lookup behavior, and 19 unit
tests covering canonical loading, pseudo-locale behavior, placeholder
locales, malformed input, and parser edge cases.

**Gate status: OPEN — blocked on Q7 human translation work for
non-English locales.**  The repository now contains the required locale
files, but `fr.yaml`, `de.yaml`, `es.yaml`, `ru.yaml`, `pl.yaml`, and
`it.yaml` intentionally remain placeholders pending human translators
(see Q7 in `docs/questions.md`).

Workspace is clean under fmt, clippy, and test.

## Gate evidence

| Requirement | Status |
|---|---|
| Seven locale files exist under `locales/` | ✅ `en`, `zz`, `fr`, `de`, `es`, `ru`, `pl`, `it` |
| English canonical locale authored | ✅ `locales/en.yaml` |
| Pseudo-locale `zz` bracket-wraps all EN strings | ✅ `locales/zz.yaml` + tests |
| Non-EN human locales represented with TODO placeholders | ✅ Six placeholder YAML files referencing Q7 |
| Locale loader uses deterministic ordered storage | ✅ `BTreeMap<String, String>` |
| No `HashMap` used | ✅ Confirmed in `locale.rs` |
| No floats used | ✅ Integer/string-only parsing |
| 15+ tests added | ✅ 19 tests in `crates/client-shared/src/locale.rs` |
| Unknown localization key falls back to key | ✅ Covered by unit tests |

## What was built

### `locales/`

- `en.yaml` — canonical English source with 43 UI strings
- `zz.yaml` — pseudo-locale with bracket-wrapped values
- `fr.yaml`, `de.yaml`, `es.yaml`, `ru.yaml`, `pl.yaml`, `it.yaml` —
  placeholder locale files with TODO markers tied to Q7

### `gc1805-client-shared`

- `src/locale.rs`
  - `Locale { id, strings }`
  - `Locale::get(&self, key) -> &str` fallback behavior
  - `Locale::load_yaml(&str) -> Result<Locale, String>` manual parser for
    the simple project locale-file subset
  - 19 unit tests
- `src/lib.rs`
  - exports `pub mod locale;`

## ⚠ Gate blocker

**Q7 — Locale translators.**  PROMPT.md §14 requires seven locales, but
only `en` and `zz` can be authored in this phase without human
translation input. The six non-English production locales are blocked on
human translators and are deliberately left as placeholders:
`fr`, `de`, `es`, `ru`, `pl`, `it`.

## Hard rules compliance

- ✅ No floats
- ✅ No `HashMap`
- ✅ 15+ tests (19 added)
- ✅ Deterministic ordered storage via `BTreeMap`
- ✅ Explicitly documented Q7 gate blocker
