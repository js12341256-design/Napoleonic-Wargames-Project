# Phase 3 — Economy

Date closed: 2026-04-25
Branch: `claude/implement-design-system-ciX1R`
Gate: `docs/PROMPT.md` §16.4

## Summary

Economic phase resolver covering the full turn loop: income, maintenance,
replacement queue, production queue, and subsidy transfers.  Order
vocabulary extended with `SetTaxPolicy`, `BuildCorps`, `BuildFleet`, and
`Subsidize`.  22 hand-written test cases, headless CLI
`gc1805 economic-phase`, and schema regeneration.

92 tests passing (70 prior + 22 new economy tests).  Workspace clean
under fmt, clippy, build, test.

## Gate evidence

| §16.4 requirement | Status |
|---|---|
| Income across ownership, tax-policy, blockade variations | ✅ Tests 1–6 in `economy.rs::tests` |
| Maintenance for corps and fleets, deficit clamping | ✅ Tests 7–9 |
| Replacement queue scheduling (ETA hit, miss, multiple) | ✅ Tests 10–12 |
| Production queue scheduling (corps, fleet, ETA miss) | ✅ Tests 13–15 |
| Subsidy transfer happy path | ✅ Test 16 |
| Tax policy order apply | ✅ Test 17 |
| BuildCorps validation (ok, broke) | ✅ Tests 18–19 |
| Subsidy validation (war, broke) | ✅ Tests 20–21 |
| Determinism: identical inputs → identical events | ✅ Test 22 |
| 22+ hand-written calculation cases | ✅ 22 cases in `economy.rs::tests` |
| Headless CLI `economic-phase` subcommand | ✅ `gc1805 economic-phase <scenario.json> [--tables <path>]` |

## What was built

### `gc1805-core-schema`

- `events.rs` — Seven new `Event` variants: `IncomePaid`, `MaintenancePaid`,
  `TreasuryInDeficit`, `ReplacementsArrived`, `UnitProduced`, `SubsidyTransferred`,
  `TaxPolicySet`.  Field `unit_kind` used in `UnitProduced` to avoid serde tag collision.
- `scenario::Area` — Added `blockaded: bool` field (`#[serde(default)]` for
  forward-compatible JSON).

### `gc1805-core`

- `economy.rs` — New module with:
  - `resolve_economic_phase(&mut Scenario, &EconomyTable) -> Vec<Event>` —
    full five-step economic turn: income → maintenance → replacements →
    production → subsidies.  All iteration over `power_state` (BTreeMap)
    is deterministic.  Placeholder values in the table produce identity
    behaviour (multiplier falls back to 10 000; maintenance rate skips
    deduction).
  - `validate_economic_order(&Scenario, &EconomyTable, &Order) -> Result<(), String>` —
    pure check for `SetTaxPolicy`, `BuildCorps`, `BuildFleet`, `Subsidize`.
    Non-economic orders return `Err`.
  - `apply_economic_order(&mut Scenario, &EconomyTable, &Order) -> Event` —
    mutates scenario and returns the resulting event.  Build orders emit
    `OrderRejected` with reason codes `BUILD_CORPS_QUEUED` /
    `BUILD_FLEET_QUEUED` / `SUBSIDY_QUEUED` (they are queued, not instant).
  - 22 `#[cfg(test)]` cases with `minimal_scenario()` and `standard_tables()`
    fixtures using `Maybe::Value` throughout.
- `lib.rs` — re-exports `resolve_economic_phase`, `validate_economic_order`,
  `apply_economic_order`.

### `gc1805-cli` (`gc1805` binary)

- `economic-phase <scenario.json> [--tables <economy.json>]` subcommand.
  Loads scenario (and optional economy table), runs one economic phase,
  prints events as a JSON array, then prints treasury per power.

### Documentation

- `docs/rules/economy.md` — pre-existing canonical reference; no changes needed.
- `docs/phase-reports/phase-03.md` — this file.
- `data/schemas/` — regenerated via `cargo run -p xtask -- dump-schemas`;
  `scenario.schema.json` now includes `blockaded` on `Area`.

## ADRs added

None.

## Adjudications added

None.

## Open questions

All prior open questions carry forward unchanged.  Phase 3 itself has no
new blocking questions: all numerics are typed `Maybe<i32>` and the
resolver tolerates placeholders gracefully.

## Known defects and caveats

- `apply_economic_order` for `BuildCorps` / `BuildFleet` returns
  `OrderRejected` with reason codes `BUILD_CORPS_QUEUED` etc.  This is
  intentional: orders are _queued_ rather than immediately fulfilled, so
  "rejected" here is a signal to the caller that the immediate unit spawn
  has not happened.  A cleaner approach would be a separate `OrderQueued`
  event; deferred to Phase 6 when the full order-pipeline is revisited.
- Production ID generation (`CORPS_FRA_T0_PARIS`) is stable within a
  single turn but not across turns if the same area produces two units in
  the same turn.  Phase 5 introduces depot tracking which will force unique
  IDs; this is documented in the production item logic.
- Subsidy resolution does not yet validate supply-line or treaty
  constraints (Phases 5 and 6 add those).

## Next phase

Phase 4 — Combat (`docs/PROMPT.md` §16.5).  Requires Phase-3 treasury
and manpower state as inputs.  All economic state is now live.
