//! Fog-of-war projection.
//!
//! PROMPT.md §5.4: clients receive a projection filtered by the power
//! they control.  The simulation core holds the full state; this
//! function produces what one player should see.
//!
//! # Visibility model (Phase 1 baseline)
//!
//! Phase 1 implements only the static rules that follow from the
//! scenario alone — corps and fleet positions become hidden according
//! to ownership and area control.  Later phases extend this with
//! cavalry-screening reveals (§7.5) and live diplomatic-orders fog
//! (§7 / §16.8).
//!
//! Concretely, for a viewer power `P`:
//!
//! 1. `powers`, `minors`, `leaders`, `areas`, `sea_zones`, `adjacency`,
//!    `coast_links`, `sea_adjacency` are visible in full.  None of
//!    these reveal a secret about another power.
//! 2. `diplomacy` keeps every pair where `P` is a member, plus every
//!    pair whose state is `War` (war is public).  Other pairs are
//!    omitted.
//! 3. `corps` and `fleets` keep:
//!     - all units owned by `P`,
//!     - units in an area or port owned by `P` (you can see what's
//!       sitting on top of you).
//!
//!    Otherwise they are dropped.  Their composition is therefore
//!    fully revealed when present and fully hidden when absent — the
//!    bracketed-count partial reveal lands with screening in §7.5.
//!
//! The projection is itself a `Scenario`, so it round-trips through
//! the canonical-JSON pipeline unchanged.

use gc1805_core_schema::ids::{AreaId, PowerId};
use gc1805_core_schema::scenario::{DiplomaticState, Owner, Scenario};
use std::collections::BTreeMap;

/// The viewer-specific projection of a scenario.
#[derive(Debug, Clone)]
pub struct ProjectedScenario {
    pub viewer: PowerId,
    pub view: Scenario,
}

pub fn project(full: &Scenario, viewer: &PowerId) -> ProjectedScenario {
    let mut view = full.clone();

    // ── Diplomacy: keep pairs the viewer is in, plus all WAR pairs.
    let kept = view
        .diplomacy
        .iter()
        .filter(|(k, v)| {
            let touches_viewer = &k.0 == viewer || &k.1 == viewer;
            touches_viewer || matches!(v, DiplomaticState::War)
        })
        .map(|(k, v)| (k.clone(), *v))
        .collect::<BTreeMap<_, _>>();
    view.diplomacy = kept;

    // ── Corps / fleets: keep if owner == viewer or in viewer's area.
    let viewer_areas: std::collections::BTreeSet<AreaId> = full
        .areas
        .iter()
        .filter(|(_, a)| match &a.owner {
            Owner::Power(slot) => &slot.power == viewer,
            _ => false,
        })
        .map(|(id, _)| id.clone())
        .collect();

    view.corps = full
        .corps
        .iter()
        .filter(|(_, c)| &c.owner == viewer || viewer_areas.contains(&c.area))
        .map(|(id, c)| (id.clone(), c.clone()))
        .collect();

    view.fleets = full
        .fleets
        .iter()
        .filter(|(_, f)| {
            &f.owner == viewer
                || f.at_port
                    .as_ref()
                    .map(|a| viewer_areas.contains(a))
                    .unwrap_or(false)
        })
        .map(|(id, f)| (id.clone(), f.clone()))
        .collect();

    ProjectedScenario {
        viewer: viewer.clone(),
        view,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core_schema::ids::{AreaId, CorpsId, LeaderId, PowerId};
    use gc1805_core_schema::scenario::{
        Area, Corps, DiplomaticPairKey, DiplomaticState, Features, Fleet, GameDate, Leader,
        MovementRules, Owner, PowerSetup, PowerSlot, SCHEMA_VERSION, Scenario, Terrain,
    };
    use std::collections::BTreeMap;

    /// Two-power, two-area, two-corps fixture.  Shared by every
    /// projection case.
    fn fixture() -> Scenario {
        let mut s = Scenario {
            schema_version: SCHEMA_VERSION,
            rules_version: 0,
            scenario_id: "fixture".into(),
            name: "Two-Power Fixture".into(),
            start: GameDate::new(1805, 4),
            end: GameDate::new(1805, 12),
            unplayable_in_release: true,
            features: Features::default(),
            movement_rules: MovementRules::default(),
            current_turn: 0,
            power_state: BTreeMap::new(),
            production_queue: Vec::new(),
            replacement_queue: Vec::new(),
            subsidy_queue: Vec::new(),
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
        };
        s.leaders.insert(
            LeaderId::from("LEADER_N"),
            Leader {
                display_name: "N".into(),
                strategic: 5,
                tactical: 5,
                initiative: 9,
                army_commander: true,
                born: GameDate::new(1769, 8),
            },
        );
        s.leaders.insert(
            LeaderId::from("LEADER_F"),
            Leader {
                display_name: "F".into(),
                strategic: 3,
                tactical: 3,
                initiative: 5,
                army_commander: false,
                born: GameDate::new(1768, 1),
            },
        );
        s.areas.insert(
            AreaId::from("AREA_A"),
            Area {
                display_name: "A".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("FRA"),
                }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: gc1805_core_schema::tables::Maybe::Value(0),
                manpower_yield: gc1805_core_schema::tables::Maybe::Value(0),
                capital_of: Some(PowerId::from("FRA")),
                port: false,
                blockaded: false,
                map_x: 0,
                map_y: 0,
            },
        );
        s.areas.insert(
            AreaId::from("AREA_B"),
            Area {
                display_name: "B".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("AUS"),
                }),
                terrain: Terrain::Open,
                fort_level: 0,
                money_yield: gc1805_core_schema::tables::Maybe::Value(0),
                manpower_yield: gc1805_core_schema::tables::Maybe::Value(0),
                capital_of: Some(PowerId::from("AUS")),
                port: false,
                blockaded: false,
                map_x: 1,
                map_y: 0,
            },
        );
        s.powers.insert(
            PowerId::from("FRA"),
            PowerSetup {
                display_name: "France".into(),
                house: "B".into(),
                ruler: LeaderId::from("LEADER_N"),
                capital: AreaId::from("AREA_A"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 8,
                mobilization_areas: vec![],
                color_hex: "#000".into(),
            },
        );
        s.powers.insert(
            PowerId::from("AUS"),
            PowerSetup {
                display_name: "Austria".into(),
                house: "H".into(),
                ruler: LeaderId::from("LEADER_F"),
                capital: AreaId::from("AREA_B"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 8,
                max_depots: 6,
                mobilization_areas: vec![],
                color_hex: "#888".into(),
            },
        );
        s.corps.insert(
            CorpsId::from("CORPS_FRA_001"),
            Corps {
                display_name: "I.".into(),
                owner: PowerId::from("FRA"),
                area: AreaId::from("AREA_A"),
                infantry_sp: 22,
                cavalry_sp: 4,
                artillery_sp: 6,
                morale_q4: 9500,
                supplied: true,
                leader: Some(LeaderId::from("LEADER_N")),
            },
        );
        s.corps.insert(
            CorpsId::from("CORPS_AUS_001"),
            Corps {
                display_name: "Charles".into(),
                owner: PowerId::from("AUS"),
                area: AreaId::from("AREA_B"),
                infantry_sp: 17,
                cavalry_sp: 3,
                artillery_sp: 4,
                morale_q4: 8000,
                supplied: true,
                leader: Some(LeaderId::from("LEADER_F")),
            },
        );
        s
    }

    #[test]
    fn case_01_owner_sees_own_corps() {
        let s = fixture();
        let p = project(&s, &PowerId::from("FRA"));
        assert!(p.view.corps.contains_key(&CorpsId::from("CORPS_FRA_001")));
    }

    #[test]
    fn case_02_owner_does_not_see_enemy_corps_in_enemy_area() {
        let s = fixture();
        let p = project(&s, &PowerId::from("FRA"));
        assert!(!p.view.corps.contains_key(&CorpsId::from("CORPS_AUS_001")));
    }

    #[test]
    fn case_03_enemy_corps_in_my_area_revealed() {
        let mut s = fixture();
        // Move Austrian corps onto French capital.
        s.corps
            .get_mut(&CorpsId::from("CORPS_AUS_001"))
            .unwrap()
            .area = AreaId::from("AREA_A");
        let p = project(&s, &PowerId::from("FRA"));
        assert!(p.view.corps.contains_key(&CorpsId::from("CORPS_AUS_001")));
    }

    #[test]
    fn case_04_diplomacy_war_is_public() {
        let mut s = fixture();
        s.diplomacy.insert(
            DiplomaticPairKey::new(PowerId::from("AUS"), PowerId::from("FRA")),
            DiplomaticState::War,
        );
        // A neutral observer (FRA viewer can see anyway, but use a third
        // power to test the "war is public" branch).
        s.powers.insert(
            PowerId::from("GBR"),
            PowerSetup {
                display_name: "Britain".into(),
                house: "Hanover".into(),
                ruler: LeaderId::from("LEADER_F"),
                capital: AreaId::from("AREA_B"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 4,
                max_depots: 4,
                mobilization_areas: vec![],
                color_hex: "#822".into(),
            },
        );
        let p = project(&s, &PowerId::from("GBR"));
        assert_eq!(p.view.diplomacy.len(), 1);
    }

    #[test]
    fn case_05_friendly_pair_not_visible_to_third_party() {
        let mut s = fixture();
        s.diplomacy.insert(
            DiplomaticPairKey::new(PowerId::from("AUS"), PowerId::from("FRA")),
            DiplomaticState::Friendly,
        );
        s.powers.insert(
            PowerId::from("GBR"),
            PowerSetup {
                display_name: "Britain".into(),
                house: "Hanover".into(),
                ruler: LeaderId::from("LEADER_F"),
                capital: AreaId::from("AREA_B"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 4,
                max_depots: 4,
                mobilization_areas: vec![],
                color_hex: "#822".into(),
            },
        );
        let p = project(&s, &PowerId::from("GBR"));
        assert_eq!(p.view.diplomacy.len(), 0);
    }

    #[test]
    fn case_06_owner_sees_own_diplomacy_pair() {
        let mut s = fixture();
        s.diplomacy.insert(
            DiplomaticPairKey::new(PowerId::from("AUS"), PowerId::from("FRA")),
            DiplomaticState::Friendly,
        );
        let p = project(&s, &PowerId::from("FRA"));
        assert_eq!(p.view.diplomacy.len(), 1);
    }

    #[test]
    fn case_07_areas_always_visible() {
        let s = fixture();
        let p = project(&s, &PowerId::from("FRA"));
        assert_eq!(p.view.areas.len(), s.areas.len());
    }

    #[test]
    fn case_08_powers_always_visible() {
        let s = fixture();
        let p = project(&s, &PowerId::from("FRA"));
        assert_eq!(p.view.powers.len(), s.powers.len());
    }

    #[test]
    fn case_09_double_projection_is_idempotent() {
        let s = fixture();
        let p1 = project(&s, &PowerId::from("FRA")).view;
        let p2 = project(&p1, &PowerId::from("FRA")).view;
        let h1 = gc1805_core_schema::canonical_hash(&p1).unwrap();
        let h2 = gc1805_core_schema::canonical_hash(&p2).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn case_10_fleet_in_friendly_port_visible() {
        use gc1805_core_schema::ids::FleetId;
        let mut s = fixture();
        s.fleets.insert(
            FleetId::from("FLEET_AUS_001"),
            Fleet {
                display_name: "Adriatic".into(),
                owner: PowerId::from("AUS"),
                at_port: Some(AreaId::from("AREA_A")), // sitting in French port
                at_sea: None,
                ships_of_the_line: 6,
                frigates: 4,
                transports: 0,
                morale_q4: 8000,
                admiral: None,
                embarked_corps: Vec::new(),
            },
        );
        let p = project(&s, &PowerId::from("FRA"));
        assert!(p.view.fleets.contains_key(&FleetId::from("FLEET_AUS_001")));
    }

    #[test]
    fn case_11_corps_owned_by_viewer_in_enemy_area_visible_to_self() {
        let mut s = fixture();
        s.corps
            .get_mut(&CorpsId::from("CORPS_FRA_001"))
            .unwrap()
            .area = AreaId::from("AREA_B");
        let p = project(&s, &PowerId::from("FRA"));
        assert!(p.view.corps.contains_key(&CorpsId::from("CORPS_FRA_001")));
    }
}
