mod app_state;

use app_state::AppState;
use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
    GameBoard,
}

#[derive(Resource, Default)]
struct GameScenario {
    scenario_json: Option<String>,
}

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct GameBoardRoot;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.18)),
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Grand Campaign 1805"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press SPACE to start"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn teardown_main_menu(mut commands: Commands, q: Query<Entity, With<MainMenuRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn handle_main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next.set(GameState::GameBoard);
    }
}

fn setup_game_board(mut commands: Commands, scenario: Res<GameScenario>) {
    let _ = &scenario.scenario_json;
    let _ = AppState::GameBoard;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.25, 0.15)),
            GameBoardRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Board — Phase 19 adds map rendering"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Grand Campaign 1805".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<GameScenario>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(OnExit(GameState::MainMenu), teardown_main_menu)
        .add_systems(
            Update,
            handle_main_menu_input.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnEnter(GameState::GameBoard), setup_game_board)
        .run();
}
