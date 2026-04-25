//! Web client glue (Rust→WASM) for Phase 15.
//! See `docs/PROMPT.md` §16.15 and `docs/rules/ui_web.md`.
#![forbid(unsafe_code)]

use gc1805_core::economy::resolve_economic_phase;
use gc1805_core_schema::scenario::Scenario;
use gc1805_core_schema::tables::EconomyTable;
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better WASM error messages.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

pub mod helpers {
    use gc1805_core_schema::ids::{AreaId, PowerId};
    use gc1805_core_schema::scenario::Scenario;

    pub fn scenario_from_json(json: &str) -> Result<Scenario, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn scenario_to_json(scenario: &Scenario) -> String {
        serde_json::to_string(scenario).unwrap_or_default()
    }

    pub fn get_power_ids(scenario: &Scenario) -> Vec<String> {
        scenario
            .powers
            .keys()
            .map(|power_id| power_id.as_str().to_owned())
            .collect()
    }

    pub fn get_area_ids(scenario: &Scenario) -> Vec<String> {
        scenario
            .areas
            .keys()
            .map(|area_id| area_id.as_str().to_owned())
            .collect()
    }

    pub fn get_treasury(scenario: &Scenario, power_id: &str) -> i64 {
        scenario
            .power_state
            .get(&PowerId::from(power_id))
            .map(|state| state.treasury)
            .unwrap_or(0)
    }

    pub fn current_turn(scenario: &Scenario) -> u32 {
        scenario.current_turn
    }

    pub fn contains_power(scenario: &Scenario, power_id: &str) -> bool {
        scenario.powers.contains_key(&PowerId::from(power_id))
    }

    pub fn contains_area(scenario: &Scenario, area_id: &str) -> bool {
        scenario.areas.contains_key(&AreaId::from(area_id))
    }
}

/// Parse a scenario JSON string and return a handle.
#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmGame {
    scenario: Scenario,
}

#[wasm_bindgen]
impl WasmGame {
    /// Load a scenario from JSON string.
    #[wasm_bindgen(constructor)]
    pub fn new(scenario_json: &str) -> Result<WasmGame, JsValue> {
        let scenario = helpers::scenario_from_json(scenario_json)
            .map_err(|error| JsValue::from_str(&error))?;
        Ok(WasmGame { scenario })
    }

    /// Get current turn number.
    #[wasm_bindgen]
    pub fn current_turn(&self) -> u32 {
        helpers::current_turn(&self.scenario)
    }

    /// Get treasury for a power (by power ID string).
    #[wasm_bindgen]
    pub fn get_treasury(&self, power_id: &str) -> i64 {
        helpers::get_treasury(&self.scenario, power_id)
    }

    /// Get scenario as JSON string (for JS consumption).
    #[wasm_bindgen]
    pub fn to_json(&self) -> String {
        helpers::scenario_to_json(&self.scenario)
    }

    /// Run one economic phase with default placeholder tables.
    #[wasm_bindgen]
    pub fn run_economic_phase(&mut self) -> String {
        let tables = EconomyTable::default();
        let events = resolve_economic_phase(&mut self.scenario, &tables);
        serde_json::to_string(&events).unwrap_or_default()
    }

    /// Get list of power IDs as a JSON array.
    #[wasm_bindgen]
    pub fn get_power_ids(&self) -> String {
        serde_json::to_string(&helpers::get_power_ids(&self.scenario)).unwrap_or_default()
    }

    /// Get list of area IDs as a JSON array.
    #[wasm_bindgen]
    pub fn get_area_ids(&self) -> String {
        serde_json::to_string(&helpers::get_area_ids(&self.scenario)).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use gc1805_core::economy::resolve_economic_phase;
    use gc1805_core_schema::ids::PowerId;
    use gc1805_core_schema::tables::EconomyTable;
    use serde_json::{json, Value};

    fn minimal_scenario_json() -> String {
        json!({
            "schema_version": 1,
            "rules_version": 0,
            "scenario_id": "phase15_test",
            "name": "Phase 15 Test Scenario",
            "start": { "year": 1805, "month": 4 },
            "end": { "year": 1815, "month": 12 },
            "unplayable_in_release": true,
            "features": {},
            "movement_rules": {
                "max_corps_per_area": { "_placeholder": true },
                "movement_hops_per_turn": { "_placeholder": true },
                "forced_march_extra_hops": { "_placeholder": true },
                "forced_march_morale_loss_q4": { "_placeholder": true }
            },
            "power_state": {
                "FRA": {
                    "treasury": 125,
                    "manpower": 50,
                    "prestige": 7,
                    "tax_policy": "STANDARD"
                }
            },
            "production_queue": [],
            "replacement_queue": [],
            "subsidy_queue": [],
            "powers": {
                "FRA": {
                    "display_name": "France",
                    "house": "Bonaparte",
                    "ruler": "LEADER_NAPOLEON",
                    "capital": "AREA_PARIS",
                    "starting_treasury": 125,
                    "starting_manpower": 50,
                    "starting_pp": 7,
                    "max_corps": 10,
                    "max_depots": 3,
                    "mobilization_areas": ["AREA_PARIS"],
                    "color_hex": "#2a3a6a"
                }
            },
            "minors": {},
            "leaders": {},
            "areas": {
                "AREA_PARIS": {
                    "display_name": "Paris",
                    "owner": {
                        "kind": "POWER",
                        "power": "FRA"
                    },
                    "terrain": "URBAN",
                    "fort_level": 2,
                    "money_yield": { "_placeholder": true },
                    "manpower_yield": { "_placeholder": true },
                    "capital_of": "FRA",
                    "port": false,
                    "blockaded": false,
                    "map_x": 100,
                    "map_y": 200
                }
            },
            "sea_zones": {},
            "corps": {},
            "fleets": {},
            "diplomacy": {},
            "adjacency": [],
            "coast_links": [],
            "sea_adjacency": []
        })
        .to_string()
    }

    fn minimal_scenario_without_current_turn_json() -> String {
        let mut value: Value = serde_json::from_str(&minimal_scenario_json()).unwrap();
        value.as_object_mut().unwrap().remove("current_turn");
        value.to_string()
    }

    #[test]
    fn parse_minimal_scenario_ok() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        assert_eq!(scenario.scenario_id, "phase15_test");
        assert_eq!(scenario.name, "Phase 15 Test Scenario");
    }

    #[test]
    fn parse_invalid_json_err() {
        let error = scenario_from_json("{not valid json}").unwrap_err();
        assert!(!error.is_empty());
    }

    #[test]
    fn round_trip_scenario() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        let round_tripped = scenario_from_json(&scenario_to_json(&scenario)).unwrap();
        assert_eq!(round_tripped.scenario_id, scenario.scenario_id);
        assert_eq!(round_tripped.current_turn, scenario.current_turn);
        assert_eq!(round_tripped.powers.len(), scenario.powers.len());
    }

    #[test]
    fn get_power_ids_from_scenario() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        assert_eq!(get_power_ids(&scenario), vec!["FRA".to_string()]);
    }

    #[test]
    fn get_treasury_known_power() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        assert_eq!(get_treasury(&scenario, "FRA"), 125);
    }

    #[test]
    fn get_treasury_unknown_power_zero() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        assert_eq!(get_treasury(&scenario, "GBR"), 0);
    }

    #[test]
    fn scenario_to_json_nonempty() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        let json = scenario_to_json(&scenario);
        assert!(!json.is_empty());
        assert!(json.contains("phase15_test"));
    }

    #[test]
    fn get_area_ids_from_scenario() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        assert_eq!(get_area_ids(&scenario), vec!["AREA_PARIS".to_string()]);
    }

    #[test]
    fn economic_phase_runs_on_native() {
        let mut scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        let events = resolve_economic_phase(&mut scenario, &EconomyTable::default());
        assert!(!events.is_empty());
        assert!(scenario.power_state.contains_key(&PowerId::from("FRA")));
    }

    #[test]
    fn current_turn_default_zero() {
        let scenario = scenario_from_json(&minimal_scenario_without_current_turn_json()).unwrap();
        assert_eq!(current_turn(&scenario), 0);
    }

    #[test]
    fn contains_power_detects_existing_power() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        assert!(contains_power(&scenario, "FRA"));
        assert!(!contains_power(&scenario, "AUS"));
    }

    #[test]
    fn contains_area_detects_existing_area() {
        let scenario = scenario_from_json(&minimal_scenario_json()).unwrap();
        assert!(contains_area(&scenario, "AREA_PARIS"));
        assert!(!contains_area(&scenario, "AREA_LONDON"));
    }
}
