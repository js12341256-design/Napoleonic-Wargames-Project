//! Public order-validator façade.
//!
//! Servers and clients call this crate to check an [`Order`] before
//! it enters the event log (PROMPT.md §2.4).  The actual rules-aware
//! checks live in `gc1805-core`; this crate keeps the API stable
//! across phases.
//!
//! Phase 0 was empty; Phase 2 adds movement orders.
#![forbid(unsafe_code)]

use gc1805_core::movement::{validate_order, MovementPlan, MovementRejection};
use gc1805_core::orders::Order;
use gc1805_core_schema::scenario::Scenario;

/// Single entry point.  Returns the planned outcome on success, or a
/// typed rejection.
pub fn validate(scenario: &Scenario, order: &Order) -> Result<MovementPlan, MovementRejection> {
    validate_order(scenario, order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc1805_core::orders::HoldOrder;
    use gc1805_core::{load_scenario_str, schema::ids::PowerId};

    /// Smoke: every starting French corps holds-validates against the
    /// 1805 scenario.
    #[test]
    fn fra_corps_can_hold() {
        let json = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/scenarios/1805_standard/scenario.json"
        ))
        .unwrap();
        let (scenario, _) = load_scenario_str(&json).unwrap();
        for (id, c) in &scenario.corps {
            if c.owner != PowerId::from("FRA") {
                continue;
            }
            let order = Order::Hold(HoldOrder {
                submitter: PowerId::from("FRA"),
                corps: id.clone(),
            });
            assert!(validate(&scenario, &order).is_ok(), "{id} hold failed");
        }
    }
}
