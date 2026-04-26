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

// ── Game Clock WASM bindings ──

#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmGameClock {
    clock: gc1805_core::clock::GameClock,
}

#[wasm_bindgen]
impl WasmGameClock {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameClock {
        WasmGameClock {
            clock: gc1805_core::clock::GameClock::new(),
        }
    }

    #[wasm_bindgen]
    pub fn advance_tick(&mut self) {
        self.clock.advance_tick();
    }

    #[wasm_bindgen]
    pub fn get_date(&self) -> String {
        self.clock.date_string()
    }

    #[wasm_bindgen]
    pub fn set_speed(&mut self, speed: u8) {
        self.clock.set_speed(speed);
    }

    #[wasm_bindgen]
    pub fn toggle_pause(&mut self) {
        self.clock.toggle_pause();
    }

    #[wasm_bindgen]
    pub fn is_paused(&self) -> bool {
        self.clock.paused
    }
}

// ── Marshals WASM bindings ──

#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmMarshalRegistry {
    registry: gc1805_core::marshals::MarshalRegistry,
}

#[wasm_bindgen]
impl WasmMarshalRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMarshalRegistry {
        WasmMarshalRegistry {
            registry: gc1805_core::marshals::MarshalRegistry::with_historical(),
        }
    }

    #[wasm_bindgen]
    pub fn get_marshals_json(&self) -> String {
        self.registry.to_json()
    }

    #[wasm_bindgen]
    pub fn assign_marshal(&mut self, marshal_id: u32, corps_id: u32) -> Result<(), JsValue> {
        let mid = gc1805_core_schema::ids::MarshalId::from(format!("MARSHAL_{marshal_id}"));
        let cid = gc1805_core_schema::ids::CorpsId::from(format!("CORPS_{corps_id}"));
        self.registry
            .assign_marshal(&mid, &cid)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Assign a marshal by string ID to a corps by string ID.
    #[wasm_bindgen]
    pub fn assign_marshal_by_name(
        &mut self,
        marshal_id: &str,
        corps_id: &str,
    ) -> Result<(), JsValue> {
        let mid = gc1805_core_schema::ids::MarshalId::from(marshal_id);
        let cid = gc1805_core_schema::ids::CorpsId::from(corps_id);
        self.registry
            .assign_marshal(&mid, &cid)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn get_power_marshals(&self, power_id: &str) -> String {
        let pid = gc1805_core_schema::ids::PowerId::from(power_id);
        self.registry.power_marshals_json(&pid)
    }
}

// ── Division Designer WASM bindings ──

#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmDivisionRegistry {
    registry: gc1805_core::division::DivisionRegistry,
}

#[wasm_bindgen]
impl WasmDivisionRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmDivisionRegistry {
        WasmDivisionRegistry {
            registry: gc1805_core::division::DivisionRegistry::with_defaults(),
        }
    }

    #[wasm_bindgen]
    pub fn get_division_templates_json(&self) -> String {
        self.registry.to_json()
    }

    #[wasm_bindgen]
    pub fn create_division_template(&mut self, json: String) -> Result<String, JsValue> {
        self.registry
            .create_from_json(&json)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn get_division_stats(&self, template_id: &str) -> Result<String, JsValue> {
        let tid = gc1805_core_schema::ids::DivisionTemplateId::from(template_id);
        self.registry
            .stats_json(&tid)
            .map_err(|e| JsValue::from_str(&e))
    }
}

<<<<<<< HEAD
// ── Economy Registry WASM bindings ──

#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmEconomyRegistry {
    registry: gc1805_core::production::EconomyRegistry,
}

#[wasm_bindgen]
impl WasmEconomyRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEconomyRegistry {
        WasmEconomyRegistry {
            registry: gc1805_core::production::default_economies(),
        }
    }

    /// Get all economies as JSON.
    #[wasm_bindgen]
    pub fn get_economies_json(&self) -> String {
        gc1805_core::production::economies_to_json(&self.registry)
    }

    /// Get a single power's economy as JSON.
    #[wasm_bindgen]
    pub fn get_power_economy_json(&self, power_id: &str) -> String {
        let pid = gc1805_core_schema::ids::PowerId::from(power_id);
        gc1805_core::production::power_economy_to_json(&self.registry, &pid)
    }

    /// Advance all economies by N days.
    #[wasm_bindgen]
    pub fn advance_all_economies(&mut self, days: u32) {
        gc1805_core::production::advance_all_economies(&mut self.registry, days);
    }

    /// Recruit a unit: spend manpower and gold. Returns true on success.
    #[wasm_bindgen]
    pub fn recruit_unit(&mut self, power_id: &str, manpower_cost: u32, gold_cost: u32) -> bool {
        let pid = gc1805_core_schema::ids::PowerId::from(power_id);
        if let Some(economy) = self.registry.get_mut(&pid) {
            gc1805_core::production::spend_resources(economy, manpower_cost, gold_cost).is_ok()
        } else {
            false
        }
    }
=======
// ── Historical Events WASM bindings ──

#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmEventRegistry {
    registry: gc1805_core::events::EventRegistry,
    /// IDs of events whose triggers have been met (pending player resolution).
    pending_ids: Vec<u32>,
}

#[wasm_bindgen]
impl WasmEventRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEventRegistry {
        WasmEventRegistry {
            registry: gc1805_core::events::EventRegistry::with_historical(),
            pending_ids: Vec::new(),
        }
    }

    /// Check date-based triggers and update the pending list.
    #[wasm_bindgen]
    pub fn advance_event_triggers(&mut self, current_day: u32, current_month: u8, current_year: u16) {
        let new_pending = self.registry.advance_triggers(current_year, current_month, current_day as u8);
        for id in new_pending {
            if !self.pending_ids.contains(&id) {
                self.pending_ids.push(id);
            }
        }
    }

    /// Get pending events for a power as JSON.
    #[wasm_bindgen]
    pub fn get_pending_events_json(&self, power_id: &str) -> String {
        let pid = gc1805_core_schema::ids::PowerId::from(power_id);
        self.registry.pending_events_json(&pid, &self.pending_ids)
    }

    /// Player picks an option. Returns JSON array of effect summaries.
    #[wasm_bindgen]
    pub fn resolve_event(&mut self, event_id: u32, option_index: u8) -> Result<String, JsValue> {
        let effects = self
            .registry
            .resolve(event_id, option_index)
            .map_err(|e| JsValue::from_str(&e))?;
        // Remove from pending
        self.pending_ids.retain(|&id| id != event_id);
        // Return effect summaries as JSON
        let summaries: Vec<String> = effects
            .iter()
            .map(|e| match e {
                gc1805_core::events::EventEffect::ManpowerChange(n) => {
                    if *n >= 0 { format!("+{n} manpower") } else { format!("{n} manpower") }
                }
                gc1805_core::events::EventEffect::TreasuryChange(n) => {
                    if *n >= 0 { format!("+{n} treasury") } else { format!("{n} treasury") }
                }
                gc1805_core::events::EventEffect::RelationChange { power, delta } => {
                    let sign = if *delta >= 0 { "+" } else { "" };
                    format!("{sign}{delta} relations with {}", power.as_str())
                }
                gc1805_core::events::EventEffect::DeclareWar(p) => {
                    format!("War declared on {}", p.as_str())
                }
                gc1805_core::events::EventEffect::PeaceOffer(p) => {
                    format!("Peace offered to {}", p.as_str())
                }
                gc1805_core::events::EventEffect::AddTrait(m, t) => {
                    format!("{:?} added to {}", t, m.as_str())
                }
                gc1805_core::events::EventEffect::NewsMessage(msg) => msg.clone(),
            })
            .collect();
        serde_json::to_string(&summaries).map_err(|e| JsValue::from_str(&e.to_string()))
    }
>>>>>>> origin/feat/events-system
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
