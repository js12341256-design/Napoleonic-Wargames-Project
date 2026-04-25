//! Integration test: load `data/scenarios/1805_standard/scenario.json` from
//! disk, run validation, and prove canonical-JSON round-trip.

use gc1805_core::{load_scenario_str, project};
use gc1805_core_schema::canonical_hash;
use gc1805_core_schema::ids::PowerId;

fn scenario_text() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/scenarios/1805_standard/scenario.json"
    );
    std::fs::read_to_string(path).expect("scenario file must be readable")
}

#[test]
fn loads_with_no_hard_errors() {
    let json = scenario_text();
    let (scenario, report) = load_scenario_str(&json).expect("loads cleanly");
    // Placeholders are allowed in Phase 1 — but the flag must be set.
    assert!(scenario.unplayable_in_release);
    assert!(
        !report.placeholder_paths.is_empty(),
        "placeholders are expected at Phase 1"
    );
}

#[test]
fn integrity_check_clean_for_known_refs() {
    let json = scenario_text();
    let (_, report) = load_scenario_str(&json).expect("loads cleanly");
    assert!(
        report.integrity.is_empty(),
        "integrity issues: {:#?}",
        report.integrity
    );
}

#[test]
fn round_trip_canonical_hash_stable() {
    let json = scenario_text();
    let (scenario, _) = load_scenario_str(&json).expect("loads cleanly");
    let h1 = canonical_hash(&scenario).unwrap();
    let canon = gc1805_core_schema::to_canonical_string(&scenario).unwrap();
    let (scenario2, _) = load_scenario_str(&canon).expect("loads from canonical form");
    let h2 = canonical_hash(&scenario2).unwrap();
    assert_eq!(h1, h2, "canonical hash must be stable across re-load");
}

#[test]
fn france_can_see_its_own_corps_in_1805() {
    let json = scenario_text();
    let (scenario, _) = load_scenario_str(&json).expect("loads");
    let projected = project(&scenario, &PowerId::from("FRA"));
    let fra_corps = projected
        .view
        .corps
        .values()
        .filter(|c| c.owner.as_str() == "FRA")
        .count();
    assert!(
        fra_corps >= 2,
        "France should see at least its own corps; got {fra_corps}"
    );
}

#[test]
fn britain_does_not_see_napoleons_corps_in_paris() {
    let json = scenario_text();
    let (scenario, _) = load_scenario_str(&json).expect("loads");
    let projected = project(&scenario, &PowerId::from("GBR"));
    let sees_napoleon = projected
        .view
        .corps
        .values()
        .any(|c| c.leader.as_ref().map(|l| l.as_str()) == Some("LEADER_NAPOLEON"));
    assert!(
        !sees_napoleon,
        "Britain must not see Napoleon's corps in Paris"
    );
}

#[test]
fn france_britain_war_visible_to_all() {
    let json = scenario_text();
    let (scenario, _) = load_scenario_str(&json).expect("loads");
    for viewer in ["AUS", "PRU", "RUS", "SPA", "OTT"] {
        let p = project(&scenario, &PowerId::from(viewer));
        let saw_war = p
            .view
            .diplomacy
            .values()
            .any(|s| matches!(s, gc1805_core_schema::scenario::DiplomaticState::War));
        assert!(saw_war, "viewer {viewer} should see the FRA–GBR war");
    }
}
