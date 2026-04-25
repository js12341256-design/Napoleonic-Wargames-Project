//! Political phase resolver (PROMPT.md §16.7, `docs/rules/political.md`).
//!
//! Four public entry points:
//!
//! - [`apply_pp_delta`] — add/subtract prestige points for a power.
//! - [`check_revolts`] — scan areas for revolt conditions.
//! - [`check_abdication`] — scan powers for abdication conditions.
//! - [`resolve_political_phase`] — run the full political phase.
//!
//! HARD RULES (PROMPT.md §0):
//! - No floats.
//! - No wall-clock time.
//! - No HashMap in simulation logic.
//! - Designer-authored numerics stay `Maybe::Placeholder` until authored.

use gc1805_core_schema::events::Event;
use gc1805_core_schema::ids::PowerId;
use gc1805_core_schema::scenario::{Owner, Scenario};
use gc1805_core_schema::tables::{Maybe, PpModifiersTable};

/// Structural placeholder: revolt threshold for prestige.  Areas with
/// `manpower_yield > 0` whose owner's prestige is below this value
/// trigger revolts.  The real threshold must come from the designer;
/// this constant exists only so the code compiles and tests run.
const REVOLT_PRESTIGE_THRESHOLD: i32 = 0;

/// Structural placeholder: abdication threshold.  If a power's prestige
/// drops below this value, abdication is forced.  Designer must replace
/// this with a real value.
const ABDICATION_PRESTIGE_THRESHOLD: i32 = -50;

// ─── Public entry points ───────────────────────────────────────────────

/// Apply a prestige-point delta to a power.  If `tables.events` contains
/// a matching key for `reason` with `Maybe::Value`, that table value
/// overrides the passed `delta`.  Returns a `PrestigeAwarded` event.
pub fn apply_pp_delta(
    scenario: &mut Scenario,
    power: &PowerId,
    delta: i32,
    reason: &str,
    tables: &PpModifiersTable,
) -> Event {
    let effective_delta = match tables.events.get(reason) {
        Some(Maybe::Value(table_delta)) => *table_delta,
        _ => delta,
    };

    if let Some(ps) = scenario.power_state.get_mut(power) {
        ps.prestige += effective_delta;
    }

    Event::PrestigeAwarded {
        power: power.clone(),
        delta: effective_delta,
        reason: reason.to_owned(),
    }
}

/// Scan all areas in BTreeMap order.  For each area with a positive
/// `manpower_yield` (non-placeholder) owned by a power whose prestige
/// is below `REVOLT_PRESTIGE_THRESHOLD`, emit `RevoltTriggered`.
pub fn check_revolts(scenario: &Scenario) -> Vec<Event> {
    let mut events = Vec::new();
    for (_area_id, area) in &scenario.areas {
        // Only areas with known positive manpower yield.
        let has_manpower = matches!(area.manpower_yield, Maybe::Value(y) if y > 0);
        if !has_manpower {
            continue;
        }
        if let Owner::Power(slot) = &area.owner {
            if let Some(ps) = scenario.power_state.get(&slot.power) {
                if ps.prestige < REVOLT_PRESTIGE_THRESHOLD {
                    events.push(Event::RevoltTriggered {
                        area: _area_id.clone(),
                        owner: slot.power.clone(),
                    });
                }
            }
        }
    }
    events
}

/// Scan all powers in BTreeMap order.  If a power's prestige is below
/// `ABDICATION_PRESTIGE_THRESHOLD`, emit `AbdicationForced`.
pub fn check_abdication(scenario: &Scenario) -> Vec<Event> {
    let mut events = Vec::new();
    for (power_id, ps) in &scenario.power_state {
        if ps.prestige < ABDICATION_PRESTIGE_THRESHOLD {
            events.push(Event::AbdicationForced {
                power: power_id.clone(),
            });
        }
    }
    events
}

/// Run the full political phase: check revolts, then check abdication.
/// The `tables` parameter is accepted for future use and to satisfy the
/// API contract; currently revolts and abdication use structural
/// placeholder thresholds rather than table lookups.
pub fn resolve_political_phase(scenario: &mut Scenario, _tables: &PpModifiersTable) -> Vec<Event> {
    let mut events = Vec::new();
    events.extend(check_revolts(scenario));
    events.extend(check_abdication(scenario));
    events
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::{AreaId, LeaderId};
    use gc1805_core_schema::scenario::{
        Area, GameDate, Owner, PowerSetup, PowerSlot, PowerState, Scenario, TaxPolicy, Terrain,
    };
    use gc1805_core_schema::tables::Maybe;
    use std::collections::BTreeMap;

    // ── Fixtures ─────────────────────────────────────────────────────

    fn fra() -> PowerId {
        PowerId::from("FRA")
    }
    fn gbr() -> PowerId {
        PowerId::from("GBR")
    }
    fn rus() -> PowerId {
        PowerId::from("RUS")
    }
    fn area_paris() -> AreaId {
        AreaId::from("AREA_PARIS")
    }
    fn area_lyon() -> AreaId {
        AreaId::from("AREA_LYON")
    }
    fn area_london() -> AreaId {
        AreaId::from("AREA_LONDON")
    }

    fn minimal_scenario() -> Scenario {
        let mut powers = BTreeMap::new();
        powers.insert(
            fra(),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: LeaderId::from("LEADER_NAPOLEON"),
                capital: area_paris(),
                starting_treasury: 100,
                starting_manpower: 50,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 3,
                mobilization_areas: vec![],
                color_hex: "#2a3a6a".into(),
            },
        );

        let mut areas = BTreeMap::new();
        areas.insert(
            area_paris(),
            Area {
                display_name: "Paris".into(),
                owner: Owner::Power(PowerSlot { power: fra() }),
                terrain: Terrain::Urban,
                fort_level: 2,
                money_yield: Maybe::Value(10),
                manpower_yield: Maybe::Value(5),
                capital_of: Some(fra()),
                port: false,
                blockaded: false,
                map_x: 0,
                map_y: 0,
            },
        );

        let mut power_state = BTreeMap::new();
        power_state.insert(
            fra(),
            PowerState {
                treasury: 100,
                manpower: 50,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );

        Scenario {
            schema_version: 1,
            rules_version: 0,
            scenario_id: "test".into(),
            name: "Test".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: true,
            features: Default::default(),
            movement_rules: Default::default(),
            current_turn: 0,
            power_state,
            production_queue: vec![],
            replacement_queue: vec![],
            subsidy_queue: vec![],
            powers,
            minors: BTreeMap::new(),
            leaders: BTreeMap::new(),
            areas,
            sea_zones: BTreeMap::new(),
            corps: BTreeMap::new(),
            fleets: BTreeMap::new(),
            diplomacy: BTreeMap::new(),
            adjacency: vec![],
            coast_links: vec![],
            sea_adjacency: vec![],
        }
    }

    fn empty_pp_tables() -> PpModifiersTable {
        PpModifiersTable {
            schema_version: 1,
            events: BTreeMap::new(),
        }
    }

    fn pp_tables_with(key: &str, value: Maybe<i32>) -> PpModifiersTable {
        let mut events = BTreeMap::new();
        events.insert(key.to_owned(), value);
        PpModifiersTable {
            schema_version: 1,
            events,
        }
    }

    // ── apply_pp_delta tests ─────────────────────────────────────────

    /// 1. Basic positive delta: prestige 0 → 10
    #[test]
    fn pp_delta_positive() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        let ev = apply_pp_delta(&mut s, &fra(), 10, "battle_won", &t);
        assert_eq!(s.power_state[&fra()].prestige, 10);
        assert!(matches!(ev, Event::PrestigeAwarded { delta: 10, .. }));
    }

    /// 2. Basic negative delta: prestige 0 → -5
    #[test]
    fn pp_delta_negative() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        let ev = apply_pp_delta(&mut s, &fra(), -5, "battle_lost", &t);
        assert_eq!(s.power_state[&fra()].prestige, -5);
        assert!(matches!(ev, Event::PrestigeAwarded { delta: -5, .. }));
    }

    /// 3. Table override: table has Value(20) for key, passed delta ignored
    #[test]
    fn pp_delta_table_override() {
        let mut s = minimal_scenario();
        let t = pp_tables_with("battle_won", Maybe::Value(20));
        let ev = apply_pp_delta(&mut s, &fra(), 5, "battle_won", &t);
        assert_eq!(s.power_state[&fra()].prestige, 20);
        assert!(matches!(ev, Event::PrestigeAwarded { delta: 20, .. }));
    }

    /// 4. Table placeholder: table has Placeholder for key, passed delta used
    #[test]
    fn pp_delta_table_placeholder_uses_passed() {
        let mut s = minimal_scenario();
        let t = pp_tables_with("battle_won", Maybe::placeholder());
        let ev = apply_pp_delta(&mut s, &fra(), 5, "battle_won", &t);
        assert_eq!(s.power_state[&fra()].prestige, 5);
        assert!(matches!(ev, Event::PrestigeAwarded { delta: 5, .. }));
    }

    /// 5. Table key missing: no match, passed delta used
    #[test]
    fn pp_delta_table_key_missing() {
        let mut s = minimal_scenario();
        let t = pp_tables_with("some_other_event", Maybe::Value(99));
        apply_pp_delta(&mut s, &fra(), 3, "battle_won", &t);
        assert_eq!(s.power_state[&fra()].prestige, 3);
    }

    /// 6. Zero delta: prestige unchanged
    #[test]
    fn pp_delta_zero() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        apply_pp_delta(&mut s, &fra(), 0, "nothing", &t);
        assert_eq!(s.power_state[&fra()].prestige, 0);
    }

    /// 7. Cumulative deltas: two successive calls
    #[test]
    fn pp_delta_cumulative() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        apply_pp_delta(&mut s, &fra(), 10, "win", &t);
        apply_pp_delta(&mut s, &fra(), -3, "loss", &t);
        assert_eq!(s.power_state[&fra()].prestige, 7);
    }

    /// 8. Unknown power: no crash, event still returned
    #[test]
    fn pp_delta_unknown_power() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        let ev = apply_pp_delta(&mut s, &gbr(), 10, "test", &t);
        // GBR not in power_state, so no mutation but no panic
        assert!(matches!(ev, Event::PrestigeAwarded { delta: 10, .. }));
    }

    /// 9. Event reason string preserved
    #[test]
    fn pp_delta_reason_preserved() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        let ev = apply_pp_delta(&mut s, &fra(), 1, "capital_captured", &t);
        if let Event::PrestigeAwarded { reason, .. } = ev {
            assert_eq!(reason, "capital_captured");
        } else {
            panic!("expected PrestigeAwarded");
        }
    }

    // ── check_revolts tests ──────────────────────────────────────────

    /// 10. No revolt when prestige >= 0
    #[test]
    fn revolt_none_when_positive_prestige() {
        let s = minimal_scenario();
        let events = check_revolts(&s);
        assert!(events.is_empty());
    }

    /// 11. Revolt triggered when prestige < 0 and area has manpower_yield > 0
    #[test]
    fn revolt_triggered_negative_prestige() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -1;
        let events = check_revolts(&s);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            Event::RevoltTriggered { area, owner }
                if *area == area_paris() && *owner == fra()
        ));
    }

    /// 12. No revolt when manpower_yield is zero
    #[test]
    fn revolt_none_zero_manpower() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -10;
        s.areas.get_mut(&area_paris()).unwrap().manpower_yield = Maybe::Value(0);
        let events = check_revolts(&s);
        assert!(events.is_empty());
    }

    /// 13. No revolt when manpower_yield is Placeholder
    #[test]
    fn revolt_none_placeholder_manpower() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -10;
        s.areas.get_mut(&area_paris()).unwrap().manpower_yield = Maybe::placeholder();
        let events = check_revolts(&s);
        assert!(events.is_empty());
    }

    /// 14. Revolt for multiple areas
    #[test]
    fn revolt_multiple_areas() {
        let mut s = minimal_scenario();
        s.areas.insert(
            area_lyon(),
            Area {
                display_name: "Lyon".into(),
                owner: Owner::Power(PowerSlot { power: fra() }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: Maybe::Value(5),
                manpower_yield: Maybe::Value(3),
                capital_of: None,
                port: false,
                blockaded: false,
                map_x: 10,
                map_y: 10,
            },
        );
        s.power_state.get_mut(&fra()).unwrap().prestige = -1;
        let events = check_revolts(&s);
        // BTreeMap order: AREA_LYON < AREA_PARIS
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], Event::RevoltTriggered { area, .. } if *area == area_lyon()));
        assert!(matches!(&events[1], Event::RevoltTriggered { area, .. } if *area == area_paris()));
    }

    /// 15. Revolt order is deterministic (BTreeMap iteration)
    #[test]
    fn revolt_deterministic_order() {
        let mut s = minimal_scenario();
        s.areas.insert(
            area_lyon(),
            Area {
                display_name: "Lyon".into(),
                owner: Owner::Power(PowerSlot { power: fra() }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: Maybe::Value(5),
                manpower_yield: Maybe::Value(3),
                capital_of: None,
                port: false,
                blockaded: false,
                map_x: 10,
                map_y: 10,
            },
        );
        s.power_state.get_mut(&fra()).unwrap().prestige = -5;
        let events1 = check_revolts(&s);
        let events2 = check_revolts(&s);
        assert_eq!(events1, events2);
    }

    /// 16. Unowned area does not trigger revolt
    #[test]
    fn revolt_none_unowned_area() {
        let mut s = minimal_scenario();
        s.areas.get_mut(&area_paris()).unwrap().owner = Owner::Unowned;
        // Even with negative prestige, unowned area should not revolt
        s.power_state.get_mut(&fra()).unwrap().prestige = -10;
        let events = check_revolts(&s);
        assert!(events.is_empty());
    }

    // ── check_abdication tests ───────────────────────────────────────

    /// 17. No abdication when prestige >= -50
    #[test]
    fn abdication_none_above_threshold() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -50;
        let events = check_abdication(&s);
        assert!(events.is_empty());
    }

    /// 18. Abdication when prestige < -50
    #[test]
    fn abdication_triggered_below_threshold() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -51;
        let events = check_abdication(&s);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            Event::AbdicationForced { power } if *power == fra()
        ));
    }

    /// 19. Abdication for multiple powers
    #[test]
    fn abdication_multiple_powers() {
        let mut s = minimal_scenario();
        s.power_state.insert(
            gbr(),
            PowerState {
                treasury: 50,
                manpower: 30,
                prestige: -100,
                tax_policy: TaxPolicy::Standard,
            },
        );
        s.power_state.get_mut(&fra()).unwrap().prestige = -60;
        let events = check_abdication(&s);
        assert_eq!(events.len(), 2);
        // BTreeMap order: FRA < GBR
        assert!(matches!(&events[0], Event::AbdicationForced { power } if *power == fra()));
        assert!(matches!(&events[1], Event::AbdicationForced { power } if *power == gbr()));
    }

    /// 20. Abdication not triggered at exactly -50
    #[test]
    fn abdication_boundary_exact_minus_50() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -50;
        assert!(check_abdication(&s).is_empty());
    }

    // ── resolve_political_phase tests ────────────────────────────────

    /// 21. Full phase with no issues: no events
    #[test]
    fn resolve_clean_scenario() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        let events = resolve_political_phase(&mut s, &t);
        assert!(events.is_empty());
    }

    /// 22. Full phase: revolt + abdication combined
    #[test]
    fn resolve_revolt_and_abdication() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -60;
        let t = empty_pp_tables();
        let events = resolve_political_phase(&mut s, &t);
        // Should have 1 revolt (Paris, manpower>0, prestige<0)
        // and 1 abdication (prestige < -50)
        let revolt_count = events
            .iter()
            .filter(|e| matches!(e, Event::RevoltTriggered { .. }))
            .count();
        let abdication_count = events
            .iter()
            .filter(|e| matches!(e, Event::AbdicationForced { .. }))
            .count();
        assert_eq!(revolt_count, 1);
        assert_eq!(abdication_count, 1);
    }

    /// 23. Full phase: revolts come before abdication in event order
    #[test]
    fn resolve_revolt_before_abdication_order() {
        let mut s = minimal_scenario();
        s.power_state.get_mut(&fra()).unwrap().prestige = -60;
        let t = empty_pp_tables();
        let events = resolve_political_phase(&mut s, &t);
        assert!(events.len() >= 2);
        assert!(matches!(&events[0], Event::RevoltTriggered { .. }));
        assert!(matches!(
            events.last().unwrap(),
            Event::AbdicationForced { .. }
        ));
    }

    /// 24. Determinism: same input → same output
    #[test]
    fn resolve_determinism() {
        let mut s1 = minimal_scenario();
        s1.power_state.get_mut(&fra()).unwrap().prestige = -60;
        let mut s2 = s1.clone();
        let t = empty_pp_tables();
        let events1 = resolve_political_phase(&mut s1, &t);
        let events2 = resolve_political_phase(&mut s2, &t);
        assert_eq!(events1, events2);
    }

    /// 25. Multi-power scenario: only affected powers emit events
    #[test]
    fn resolve_multi_power_selective() {
        let mut s = minimal_scenario();
        s.power_state.insert(
            gbr(),
            PowerState {
                treasury: 50,
                manpower: 30,
                prestige: 10,
                tax_policy: TaxPolicy::Standard,
            },
        );
        s.areas.insert(
            area_london(),
            Area {
                display_name: "London".into(),
                owner: Owner::Power(PowerSlot { power: gbr() }),
                terrain: Terrain::Urban,
                fort_level: 3,
                money_yield: Maybe::Value(15),
                manpower_yield: Maybe::Value(4),
                capital_of: Some(gbr()),
                port: true,
                blockaded: false,
                map_x: 100,
                map_y: 100,
            },
        );
        // FRA negative, GBR positive
        s.power_state.get_mut(&fra()).unwrap().prestige = -5;
        let t = empty_pp_tables();
        let events = resolve_political_phase(&mut s, &t);
        // Only FRA's Paris should revolt, not GBR's London
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            Event::RevoltTriggered { owner, .. } if *owner == fra()
        ));
    }

    /// 26. Table override applied in apply_pp_delta with negative table value
    #[test]
    fn pp_delta_table_override_negative() {
        let mut s = minimal_scenario();
        let t = pp_tables_with("disaster", Maybe::Value(-30));
        apply_pp_delta(&mut s, &fra(), -5, "disaster", &t);
        assert_eq!(s.power_state[&fra()].prestige, -30);
    }

    /// 27. Three powers, only the one below threshold abdicates
    #[test]
    fn abdication_selective_among_three() {
        let mut s = minimal_scenario();
        s.power_state.insert(
            gbr(),
            PowerState {
                treasury: 50,
                manpower: 30,
                prestige: -51,
                tax_policy: TaxPolicy::Standard,
            },
        );
        s.power_state.insert(
            rus(),
            PowerState {
                treasury: 80,
                manpower: 60,
                prestige: 5,
                tax_policy: TaxPolicy::Standard,
            },
        );
        let events = check_abdication(&s);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            Event::AbdicationForced { power } if *power == gbr()
        ));
    }

    /// 28. apply_pp_delta returns correct power in event
    #[test]
    fn pp_delta_event_power_correct() {
        let mut s = minimal_scenario();
        s.power_state.insert(
            gbr(),
            PowerState {
                treasury: 50,
                manpower: 30,
                prestige: 0,
                tax_policy: TaxPolicy::Standard,
            },
        );
        let t = empty_pp_tables();
        let ev = apply_pp_delta(&mut s, &gbr(), 7, "test", &t);
        if let Event::PrestigeAwarded { power, delta, .. } = ev {
            assert_eq!(power, gbr());
            assert_eq!(delta, 7);
        } else {
            panic!("expected PrestigeAwarded");
        }
    }

    /// 29. Revolt not triggered for minor-owned area
    #[test]
    fn revolt_none_minor_owned() {
        let mut s = minimal_scenario();
        s.areas.get_mut(&area_paris()).unwrap().owner =
            Owner::Minor(gc1805_core_schema::scenario::MinorSlot {
                minor: gc1805_core_schema::ids::MinorId::from("MINOR_BAVARIA"),
            });
        s.power_state.get_mut(&fra()).unwrap().prestige = -10;
        let events = check_revolts(&s);
        assert!(events.is_empty());
    }

    /// 30. Large prestige swing: apply_pp_delta handles large values
    #[test]
    fn pp_delta_large_values() {
        let mut s = minimal_scenario();
        let t = empty_pp_tables();
        apply_pp_delta(&mut s, &fra(), i32::MAX / 2, "huge_win", &t);
        assert_eq!(s.power_state[&fra()].prestige, i32::MAX / 2);
    }
}
