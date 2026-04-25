#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    ScenarioSelect,
    Loading,
    GameBoard,
    OrderEntry,
    EndTurn,
    Results,
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn can_transition(from: &AppState, to: &AppState) -> bool {
    matches!(
        (from, to),
        (AppState::MainMenu, AppState::ScenarioSelect)
            | (AppState::ScenarioSelect, AppState::Loading)
            | (AppState::Loading, AppState::GameBoard)
            | (AppState::GameBoard, AppState::OrderEntry)
            | (AppState::OrderEntry, AppState::GameBoard)
            | (AppState::GameBoard, AppState::EndTurn)
            | (AppState::EndTurn, AppState::GameBoard)
            | (AppState::EndTurn, AppState::Results)
            | (_, AppState::MainMenu)
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn state_display_name(state: &AppState) -> &'static str {
    match state {
        AppState::MainMenu => "Main Menu",
        AppState::ScenarioSelect => "Select Scenario",
        AppState::Loading => "Loading...",
        AppState::GameBoard => "Game Board",
        AppState::OrderEntry => "Enter Orders",
        AppState::EndTurn => "End Turn",
        AppState::Results => "Results",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Default)]
pub struct PendingOrders {
    pub orders: Vec<gc1805_core::orders::Order>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_to_scenario_select_ok() {
        assert!(can_transition(&AppState::MainMenu, &AppState::ScenarioSelect));
    }

    #[test]
    fn main_menu_to_game_board_invalid() {
        assert!(!can_transition(&AppState::MainMenu, &AppState::GameBoard));
    }

    #[test]
    fn game_board_to_order_entry_ok() {
        assert!(can_transition(&AppState::GameBoard, &AppState::OrderEntry));
    }

    #[test]
    fn any_to_main_menu_ok() {
        assert!(can_transition(&AppState::Results, &AppState::MainMenu));
    }

    #[test]
    fn display_main_menu() {
        assert_eq!(state_display_name(&AppState::MainMenu), "Main Menu");
    }

    #[test]
    fn display_game_board() {
        assert_eq!(state_display_name(&AppState::GameBoard), "Game Board");
    }

    #[test]
    fn default_state_is_main_menu() {
        assert_eq!(AppState::default(), AppState::MainMenu);
    }

    #[test]
    fn pending_orders_default_empty() {
        let p = PendingOrders::default();
        assert!(p.orders.is_empty());
    }

    #[test]
    fn end_turn_to_results_ok() {
        assert!(can_transition(&AppState::EndTurn, &AppState::Results));
    }

    #[test]
    fn loading_to_game_board_ok() {
        assert!(can_transition(&AppState::Loading, &AppState::GameBoard));
    }

    #[test]
    fn game_board_to_main_menu_ok() {
        assert!(can_transition(&AppState::GameBoard, &AppState::MainMenu));
    }
}
