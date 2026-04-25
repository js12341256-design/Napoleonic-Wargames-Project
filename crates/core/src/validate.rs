//! Structural integrity checks for a loaded scenario.
//!
//! Every check here is deterministic and pure.  Issues are returned
//! rather than panicking so the caller can decide whether to fail
//! load (release builds) or proceed with warnings (dev / test).

use gc1805_core_schema::ids::{validate_id, AreaId};
use gc1805_core_schema::scenario::{AreaAdjacency, MinorRelationship, Scenario};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityIssue {
    /// An ID does not match the type's required prefix or charset.
    BadId { kind: &'static str, value: String },
    /// A reference points at an entity that is not in the scenario.
    DanglingReference {
        from: String,
        to: String,
        kind: &'static str,
    },
    /// An adjacency edge is missing its reverse.
    NonSymmetricAdjacency { from: String, to: String },
    /// The `capital` of a power is not present in `areas`.
    MissingCapital { power: String, area: String },
    /// A minor's home area is not present in `areas`.
    MissingMinorHomeArea { minor: String, area: String },
    /// A unit references an area / sea zone that doesn't exist.
    UnitInUnknownLocation { unit: String, location: String },
    /// A power's ruler leader is absent from `leaders`.
    MissingRuler { power: String, leader: String },
    /// A minor is `Allied`/`Feudal`/`Conquered` but has no patron.
    MissingPatron { minor: String },
}

/// Run every structural check.  Order is fixed for deterministic
/// reporting.
pub fn validate_scenario(s: &Scenario) -> Vec<IntegrityIssue> {
    let mut out = Vec::new();
    check_id_shapes(s, &mut out);
    check_powers(s, &mut out);
    check_minors(s, &mut out);
    check_corps(s, &mut out);
    check_fleets(s, &mut out);
    check_adjacency(s, &mut out);
    out
}

fn check_id_shapes(s: &Scenario, out: &mut Vec<IntegrityIssue>) {
    fn push_bad<I: ToString>(kind: &'static str, id: &I, out: &mut Vec<IntegrityIssue>) {
        out.push(IntegrityIssue::BadId {
            kind,
            value: id.to_string(),
        });
    }
    for id in s.powers.keys() {
        if validate_id(id.as_str(), "").is_err() {
            push_bad("PowerId", id, out);
        }
    }
    for id in s.minors.keys() {
        if validate_id(id.as_str(), "MINOR_").is_err() {
            push_bad("MinorId", id, out);
        }
    }
    for id in s.leaders.keys() {
        if validate_id(id.as_str(), "LEADER_").is_err() {
            push_bad("LeaderId", id, out);
        }
    }
    for id in s.areas.keys() {
        if validate_id(id.as_str(), "AREA_").is_err() {
            push_bad("AreaId", id, out);
        }
    }
    for id in s.sea_zones.keys() {
        if validate_id(id.as_str(), "SEA_").is_err() {
            push_bad("SeaZoneId", id, out);
        }
    }
    for id in s.corps.keys() {
        if validate_id(id.as_str(), "CORPS_").is_err() {
            push_bad("CorpsId", id, out);
        }
    }
    for id in s.fleets.keys() {
        if validate_id(id.as_str(), "FLEET_").is_err() {
            push_bad("FleetId", id, out);
        }
    }
}

fn check_powers(s: &Scenario, out: &mut Vec<IntegrityIssue>) {
    for (pid, setup) in &s.powers {
        if !s.areas.contains_key(&setup.capital) {
            out.push(IntegrityIssue::MissingCapital {
                power: pid.to_string(),
                area: setup.capital.to_string(),
            });
        }
        if !s.leaders.contains_key(&setup.ruler) {
            out.push(IntegrityIssue::MissingRuler {
                power: pid.to_string(),
                leader: setup.ruler.to_string(),
            });
        }
        for ma in &setup.mobilization_areas {
            if !s.areas.contains_key(ma) {
                out.push(IntegrityIssue::DanglingReference {
                    from: pid.to_string(),
                    to: ma.to_string(),
                    kind: "mobilization_area",
                });
            }
        }
    }
}

fn check_minors(s: &Scenario, out: &mut Vec<IntegrityIssue>) {
    for (mid, m) in &s.minors {
        if matches!(
            m.initial_relationship,
            MinorRelationship::AlliedFree
                | MinorRelationship::Feudal
                | MinorRelationship::Conquered
        ) && m.patron.is_none()
        {
            out.push(IntegrityIssue::MissingPatron {
                minor: mid.to_string(),
            });
        }
        if let Some(p) = &m.patron {
            if !s.powers.contains_key(p) {
                out.push(IntegrityIssue::DanglingReference {
                    from: mid.to_string(),
                    to: p.to_string(),
                    kind: "patron",
                });
            }
        }
        for ha in &m.home_areas {
            if !s.areas.contains_key(ha) {
                out.push(IntegrityIssue::MissingMinorHomeArea {
                    minor: mid.to_string(),
                    area: ha.to_string(),
                });
            }
        }
    }
}

fn check_corps(s: &Scenario, out: &mut Vec<IntegrityIssue>) {
    for (cid, c) in &s.corps {
        if !s.powers.contains_key(&c.owner) {
            out.push(IntegrityIssue::DanglingReference {
                from: cid.to_string(),
                to: c.owner.to_string(),
                kind: "owner",
            });
        }
        if !s.areas.contains_key(&c.area) {
            out.push(IntegrityIssue::UnitInUnknownLocation {
                unit: cid.to_string(),
                location: c.area.to_string(),
            });
        }
        if let Some(l) = &c.leader {
            if !s.leaders.contains_key(l) {
                out.push(IntegrityIssue::DanglingReference {
                    from: cid.to_string(),
                    to: l.to_string(),
                    kind: "leader",
                });
            }
        }
    }
}

fn check_fleets(s: &Scenario, out: &mut Vec<IntegrityIssue>) {
    for (fid, f) in &s.fleets {
        if !s.powers.contains_key(&f.owner) {
            out.push(IntegrityIssue::DanglingReference {
                from: fid.to_string(),
                to: f.owner.to_string(),
                kind: "owner",
            });
        }
        match (&f.at_port, &f.at_sea) {
            (Some(area), None) => {
                if !s.areas.contains_key(area) {
                    out.push(IntegrityIssue::UnitInUnknownLocation {
                        unit: fid.to_string(),
                        location: area.to_string(),
                    });
                }
            }
            (None, Some(sea)) => {
                if !s.sea_zones.contains_key(sea) {
                    out.push(IntegrityIssue::UnitInUnknownLocation {
                        unit: fid.to_string(),
                        location: sea.to_string(),
                    });
                }
            }
            _ => {
                // Either both or neither — caught at a later phase by
                // a stricter rule; flag as dangling for now.
                out.push(IntegrityIssue::UnitInUnknownLocation {
                    unit: fid.to_string(),
                    location: "<no port/sea>".into(),
                });
            }
        }
    }
}

fn check_adjacency(s: &Scenario, out: &mut Vec<IntegrityIssue>) {
    let known_areas: BTreeSet<&AreaId> = s.areas.keys().collect();
    let mut edge_set: BTreeSet<(String, String)> = BTreeSet::new();

    for AreaAdjacency { from, to, .. } in &s.adjacency {
        if !known_areas.contains(from) {
            out.push(IntegrityIssue::DanglingReference {
                from: from.to_string(),
                to: to.to_string(),
                kind: "adjacency.from",
            });
        }
        if !known_areas.contains(to) {
            out.push(IntegrityIssue::DanglingReference {
                from: from.to_string(),
                to: to.to_string(),
                kind: "adjacency.to",
            });
        }
        edge_set.insert((from.to_string(), to.to_string()));
    }

    for (a, b) in &edge_set {
        if !edge_set.contains(&(b.clone(), a.clone())) {
            out.push(IntegrityIssue::NonSymmetricAdjacency {
                from: a.clone(),
                to: b.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::PowerId;
    use gc1805_core_schema::scenario::{
        Features, GameDate, MovementRules, PowerSetup, SCHEMA_VERSION,
    };
    use std::collections::BTreeMap;

    fn empty() -> Scenario {
        Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "smoke".into(),
            name: "Smoke".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1815, 12),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules::default(),
            powers: BTreeMap::new(),
            minors: BTreeMap::new(),
            leaders: BTreeMap::new(),
            areas: BTreeMap::new(),
            sea_zones: BTreeMap::new(),
            corps: BTreeMap::new(),
            fleets: BTreeMap::new(),
            diplomacy: BTreeMap::new(),
            adjacency: Vec::new(),
            coast_links: Vec::new(),
            sea_adjacency: Vec::new(),
        }
    }

    #[test]
    fn empty_scenario_validates_clean() {
        let s = empty();
        assert!(validate_scenario(&s).is_empty());
    }

    #[test]
    fn lowercase_power_id_is_flagged() {
        let mut s = empty();
        s.powers.insert(
            PowerId::from("fra"),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: "LEADER_NAPOLEON".into(),
                capital: "AREA_PARIS".into(),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 10,
                mobilization_areas: vec![],
                color_hex: "#000000".into(),
            },
        );
        let issues = validate_scenario(&s);
        assert!(issues.iter().any(|i| matches!(
            i,
            IntegrityIssue::BadId {
                kind: "PowerId",
                ..
            }
        )));
    }
}
