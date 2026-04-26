# Economy rules — canonical reference

Sourced from PROMPT.md §16.4, §6.1 (placeholders), §7.9 (replacement
queue), §8.2 (tax-policy confirmations), and the data shapes defined
in `gc1805_core_schema::tables::EconomyTable`.

## 1. State

Per scenario the live economic state lives in two places:

- `Scenario.power_state[power]` — current `treasury` (i64 francs),
  `manpower` (i32 SP), `prestige` (i32 PP), and `tax_policy`
  (`TaxPolicy::{Low, Standard, Heavy}`).
- `Scenario.production_queue` and `Scenario.replacement_queue` —
  ordered lists of items with their ETA turn.

`Scenario.current_turn` is the integer turn index since the scenario
start (`start.year, start.month` is turn 0).  No wall-clock time
enters the simulation; turns advance in monthly steps.

## 2. The economic phase

Runs once per turn, in this order — every step is deterministic:

1. **Income** — for each power, in BTreeMap-sorted order:
    1. `gross = sum(money_yield)` over every area whose owner is the
       power and whose `blockaded` flag is `false`.
    2. `multiplier = tax_policy_multiplier_q4[power.tax_policy]` (Q4
       fixed-point, denominator 10000).
    3. `net = (gross * multiplier) / 10000`, integer division.
    4. `power.treasury += net`.
    5. Emit `Event::IncomePaid { power, gross, net, tax_policy }`.
2. **Maintenance** — corps and fleets cost upkeep.  For each owner:
    1. Per corps, deduct `corps_maintenance_per_sp * (inf+cav+art)`.
    2. Per fleet, deduct `fleet_maintenance_per_ship * total_ships`.
    3. If treasury would go negative, record an `Event::TreasuryInDeficit`
       and clamp at 0; the next turn applies a morale penalty
       (Phase 4 wires this in).
3. **Replacements** — process every `ReplacementItem` whose
   `eta_turn == current_turn`: add `sp_amount` to `power.manpower`
   and emit `Event::ReplacementsArrived`.  Items with later ETAs
   stay in the queue.
4. **Production** — process every `ProductionItem` whose
   `eta_turn == current_turn`: spawn the corps or fleet, deduct from
   the queue, emit `Event::UnitProduced`.
5. **Subsidies** — process queued `SubsidyOrder`s in the order they
   were submitted.  Transfer `amount` from sender's treasury to
   recipient's, validating that the sender can pay.  Emit
   `Event::SubsidyTransferred`.

## 3. Manpower regeneration (§7.9)

Each turn, a Q12 fraction of the **previous turn's combat losses**
is restored as a `ReplacementItem` with
`eta_turn = current_turn + recovery_lag_turns`.  Phase 3 ships the
queue mechanism; Phase 4 (combat) generates the `combat_losses`
input.  Until then, replacements only fire if the test fixture
inserts them directly.

## 4. Tax policy (§8.2)

Three levels — `Low`, `Standard`, `Heavy`.  Each has a designer-
authored Q4 multiplier in `EconomyTable.tax_policy_multipliers`.
Setting `Heavy` requires a UI confirmation per §8.2; that lives in
the client, not in the validator.

Setting tax policy is a free action and takes effect at the next
economic phase.  Storing the new policy in `power.tax_policy` is the
entire effect — multipliers are read at income time.

## 5. Production (§7.2)

A `BuildCorps` order:

- Must be issued in the power's capital, an explicit
  `mobilization_area`, or a friendly depot (Phase 5 adds depots).
- Costs `economy.corps_build_cost_money` francs and
  `economy.corps_build_cost_manpower` SP, both deducted at order
  time.  Validation rejects the order if the power can't pay.
- ETA is `current_turn + economy.corps_production_lag_months`.
- The new corps starts with `economy.new_corps_morale_q4` morale
  (representing rawness).

Fleet production is the same shape with `fleet_*` keys.  Phase 9
extends with naval-only constraints.

## 6. Subsidies

A `SubsidyOrder { from, to, amount }` is queued during the diplomatic
phase (Phase 6) and applied at the next economic phase's subsidy
step.  Validation: `amount > 0`, sender has the money, recipient is
not at war with the sender.  The actual transfer happens during
resolution; until then it is a pending order.

## 7. Forbidden

- No floats anywhere.
- Tax-policy multipliers, maintenance rates, and production lags are
  designer-authored.  Phase 3 ships them as `Maybe::Placeholder`.
- Subsidy validation does not check supply lines or treaties yet —
  Phases 5 and 6 add those checks.

## 8. Test coverage

`crates/core/src/economy.rs::tests` ships with at least 20 hand-
written cases per the §16.4 gate, covering:

- Income across ownership variations, tax-policy variations, blockade
  flag.
- Maintenance for corps and fleets with varied compositions.
- Production queue scheduling (ETA hits, ETA misses, multiple per
  turn, deterministic ordering).
- Manpower replacement queue.
- Subsidies (happy path, insufficient funds, war-state veto).
- A determinism test: given identical inputs, treasury state hash is
  identical.
