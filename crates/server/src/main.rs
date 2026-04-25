//! Authoritative server for Grand Campaign 1805.
//! Phase 12 implementation: in-memory axum HTTP + WebSocket scaffold.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use gc1805_core::orders::Order;
use gc1805_core::projection::project;
use gc1805_core_schema::{
    events::Event,
    ids::PowerId,
    scenario::{Features, GameDate, MovementRules, Scenario, SCHEMA_VERSION},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

static NEXT_GAME_ID: AtomicU64 = AtomicU64::new(1);

/// Game session state.
#[derive(Debug, Clone)]
pub struct GameSession {
    pub game_id: String,
    pub scenario: Scenario,
    pub event_log: Vec<Event>,
    pub players: BTreeMap<String, PowerId>,
}

/// Server state shared by all handlers.
#[derive(Debug)]
pub struct AppState {
    pub sessions: Mutex<BTreeMap<String, GameSession>>,
}

type SharedState = Arc<AppState>;

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateGameResponse {
    game_id: String,
}

#[derive(Debug, Deserialize)]
struct StateQuery {
    player_id: String,
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    #[serde(default)]
    since: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SubmitOrdersRequest {
    player_id: String,
    orders: Vec<Order>,
}

#[derive(Debug, Deserialize)]
struct ReconnectQuery {
    #[serde(default)]
    last_event_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubmitOrdersResponse {
    accepted: usize,
    event_log_len: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
        .into_response()
}

fn default_scenario(game_id: &str) -> Scenario {
    Scenario {
        schema_version: SCHEMA_VERSION,
        rules_version: 0,
        scenario_id: format!("server_{game_id}"),
        name: format!("Server Session {game_id}"),
        start: GameDate::new(1805, 4),
        end: GameDate::new(1815, 12),
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
    }
}

fn next_game_id() -> String {
    let id = NEXT_GAME_ID.fetch_add(1, Ordering::Relaxed);
    format!("game-{id:08}")
}

pub fn app(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/games", post(create_game))
        .route("/games/{game_id}/state", get(get_state))
        .route("/games/{game_id}/orders", post(submit_orders))
        .route("/games/{game_id}/events", get(get_events))
        .route("/games/{game_id}/ws", get(game_ws))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
    })
}

async fn create_game(State(state): State<SharedState>) -> Response {
    let game_id = next_game_id();
    let session = GameSession {
        game_id: game_id.clone(),
        scenario: default_scenario(&game_id),
        event_log: Vec::new(),
        players: BTreeMap::new(),
    };

    let mut sessions = state
        .sessions
        .lock()
        .expect("server state mutex poisoned while creating game");
    sessions.insert(game_id.clone(), session);

    (StatusCode::CREATED, Json(CreateGameResponse { game_id })).into_response()
}

async fn get_state(
    State(state): State<SharedState>,
    Path(game_id): Path<String>,
    Query(query): Query<StateQuery>,
) -> Response {
    let sessions = state
        .sessions
        .lock()
        .expect("server state mutex poisoned while reading state");
    let Some(session) = sessions.get(&game_id) else {
        return error_response(
            StatusCode::NOT_FOUND,
            format!("unknown game_id `{game_id}`"),
        );
    };

    let Some(power) = session.players.get(&query.player_id) else {
        return error_response(
            StatusCode::NOT_FOUND,
            format!(
                "unknown player_id `{}` for game `{game_id}`",
                query.player_id
            ),
        );
    };

    let projected = project(&session.scenario, power);
    Json(projected.view).into_response()
}

async fn submit_orders(
    State(state): State<SharedState>,
    Path(game_id): Path<String>,
    Json(request): Json<SubmitOrdersRequest>,
) -> Response {
    let mut sessions = state
        .sessions
        .lock()
        .expect("server state mutex poisoned while submitting orders");
    let Some(session) = sessions.get_mut(&game_id) else {
        return error_response(
            StatusCode::NOT_FOUND,
            format!("unknown game_id `{game_id}`"),
        );
    };

    let Some(expected_power) = session.players.get(&request.player_id) else {
        return error_response(
            StatusCode::NOT_FOUND,
            format!(
                "unknown player_id `{}` for game `{game_id}`",
                request.player_id
            ),
        );
    };

    for order in &request.orders {
        if order.submitter() != expected_power {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "player `{}` controls `{}` but submitted order for `{}`",
                    request.player_id,
                    expected_power,
                    order.submitter()
                ),
            );
        }
    }

    Json(SubmitOrdersResponse {
        accepted: request.orders.len(),
        event_log_len: session.event_log.len(),
    })
    .into_response()
}

async fn get_events(
    State(state): State<SharedState>,
    Path(game_id): Path<String>,
    Query(query): Query<EventsQuery>,
) -> Response {
    let sessions = state
        .sessions
        .lock()
        .expect("server state mutex poisoned while reading events");
    let Some(session) = sessions.get(&game_id) else {
        return error_response(
            StatusCode::NOT_FOUND,
            format!("unknown game_id `{game_id}`"),
        );
    };

    let events = if query.since >= session.event_log.len() {
        Vec::new()
    } else {
        session.event_log[query.since..].to_vec()
    };

    Json(events).into_response()
}

async fn game_ws(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
    Path(game_id): Path<String>,
    Query(query): Query<ReconnectQuery>,
) -> Response {
    let replay_events = {
        let sessions = state
            .sessions
            .lock()
            .expect("server state mutex poisoned while opening websocket");
        let Some(session) = sessions.get(&game_id) else {
            return error_response(
                StatusCode::NOT_FOUND,
                format!("unknown game_id `{game_id}`"),
            );
        };

        if query.last_event_index >= session.event_log.len() {
            Vec::new()
        } else {
            session.event_log[query.last_event_index..].to_vec()
        }
    };

    ws.on_upgrade(move |socket| websocket_session(socket, replay_events))
        .into_response()
}

async fn websocket_session(mut socket: WebSocket, replay_events: Vec<Event>) {
    let connected = json!({ "type": "connected" }).to_string();
    if socket.send(Message::Text(connected.into())).await.is_err() {
        return;
    }

    for event in replay_events {
        let payload = match serde_json::to_string(&event) {
            Ok(payload) => payload,
            Err(_) => return,
        };
        if socket.send(Message::Text(payload.into())).await.is_err() {
            return;
        }
    }

    while socket.recv().await.is_some() {}
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        sessions: Mutex::new(BTreeMap::new()),
    });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("bind 0.0.0.0:3000");
    axum::serve(listener, app(state))
        .await
        .expect("serve axum application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{header, Method, Request},
    };
    use futures_util::StreamExt;
    use gc1805_core::orders::{HoldOrder, MoveOrder};
    use gc1805_core_schema::{
        events::{Event, OrderRejected},
        ids::{AreaId, CorpsId, LeaderId},
        scenario::{Area, Corps, Leader, Owner, PowerSetup, PowerSlot, Terrain},
        tables::Maybe,
    };
    use tower::util::ServiceExt;

    fn test_state() -> SharedState {
        Arc::new(AppState {
            sessions: Mutex::new(BTreeMap::new()),
        })
    }

    fn scenario_fixture() -> Scenario {
        let mut scenario = default_scenario("fixture");

        scenario.leaders.insert(
            LeaderId::from("LEADER_NAPOLEON"),
            Leader {
                display_name: "Napoleon".into(),
                strategic: 6,
                tactical: 6,
                initiative: 6,
                army_commander: true,
                born: GameDate::new(1769, 8),
            },
        );
        scenario.leaders.insert(
            LeaderId::from("LEADER_CHARLES"),
            Leader {
                display_name: "Charles".into(),
                strategic: 5,
                tactical: 5,
                initiative: 4,
                army_commander: true,
                born: GameDate::new(1771, 9),
            },
        );
        scenario.powers.insert(
            PowerId::from("FRA"),
            PowerSetup {
                display_name: "France".into(),
                house: "Bonaparte".into(),
                ruler: LeaderId::from("LEADER_NAPOLEON"),
                capital: AreaId::from("AREA_PARIS"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 12,
                max_depots: 8,
                mobilization_areas: vec![AreaId::from("AREA_PARIS")],
                color_hex: "#0000ff".into(),
            },
        );
        scenario.powers.insert(
            PowerId::from("AUS"),
            PowerSetup {
                display_name: "Austria".into(),
                house: "Habsburg".into(),
                ruler: LeaderId::from("LEADER_CHARLES"),
                capital: AreaId::from("AREA_VIENNA"),
                starting_treasury: 0,
                starting_manpower: 0,
                starting_pp: 0,
                max_corps: 10,
                max_depots: 7,
                mobilization_areas: vec![AreaId::from("AREA_VIENNA")],
                color_hex: "#ffffff".into(),
            },
        );
        scenario.areas.insert(
            AreaId::from("AREA_PARIS"),
            Area {
                display_name: "Paris".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("FRA"),
                }),
                terrain: Terrain::Urban,
                fort_level: 2,
                money_yield: Maybe::Value(5),
                manpower_yield: Maybe::Value(2),
                capital_of: Some(PowerId::from("FRA")),
                port: false,
                blockaded: false,
                map_x: 10,
                map_y: 10,
            },
        );
        scenario.areas.insert(
            AreaId::from("AREA_VIENNA"),
            Area {
                display_name: "Vienna".into(),
                owner: Owner::Power(PowerSlot {
                    power: PowerId::from("AUS"),
                }),
                terrain: Terrain::Urban,
                fort_level: 2,
                money_yield: Maybe::Value(5),
                manpower_yield: Maybe::Value(2),
                capital_of: Some(PowerId::from("AUS")),
                port: false,
                blockaded: false,
                map_x: 20,
                map_y: 20,
            },
        );
        scenario.corps.insert(
            CorpsId::from("CORPS_FRA_001"),
            Corps {
                display_name: "I Corps".into(),
                owner: PowerId::from("FRA"),
                area: AreaId::from("AREA_PARIS"),
                infantry_sp: 10,
                cavalry_sp: 2,
                artillery_sp: 1,
                morale_q4: 9000,
                supplied: true,
                leader: Some(LeaderId::from("LEADER_NAPOLEON")),
            },
        );
        scenario.corps.insert(
            CorpsId::from("CORPS_AUS_001"),
            Corps {
                display_name: "Austrian Corps".into(),
                owner: PowerId::from("AUS"),
                area: AreaId::from("AREA_VIENNA"),
                infantry_sp: 9,
                cavalry_sp: 2,
                artillery_sp: 1,
                morale_q4: 8500,
                supplied: true,
                leader: Some(LeaderId::from("LEADER_CHARLES")),
            },
        );

        scenario
    }

    fn insert_session(
        state: &SharedState,
        game_id: &str,
        scenario: Scenario,
        event_log: Vec<Event>,
        players: &[(&str, &str)],
    ) {
        let mut mapping = BTreeMap::new();
        for (player_id, power_id) in players {
            mapping.insert((*player_id).to_owned(), PowerId::from(*power_id));
        }

        state.sessions.lock().expect("test mutex poisoned").insert(
            game_id.to_owned(),
            GameSession {
                game_id: game_id.to_owned(),
                scenario,
                event_log,
                players: mapping,
            },
        );
    }

    async fn body_json<T: for<'de> Deserialize<'de>>(response: Response) -> T {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("collect body bytes");
        serde_json::from_slice(&bytes).expect("parse json body")
    }

    async fn spawn_test_server(state: SharedState) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("listener local addr");
        let handle = tokio::spawn(async move {
            axum::serve(listener, app(state))
                .await
                .expect("serve test app");
        });
        (format!("127.0.0.1:{}", addr.port()), handle)
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = app(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let payload: HealthResponse = body_json(response).await;
        assert_eq!(payload.status, "ok");
    }

    #[tokio::test]
    async fn create_game_returns_id() {
        let state = test_state();
        let app = app(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/games")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let payload: CreateGameResponse = body_json(response).await;
        assert!(payload.game_id.starts_with("game-"));
        let sessions = state.sessions.lock().unwrap();
        assert!(sessions.contains_key(&payload.game_id));
    }

    #[tokio::test]
    async fn get_state_unknown_game_404() {
        let app = app(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/missing/state?player_id=player-fra")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn submit_orders_accepted() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            Vec::new(),
            &[("player-fra", "FRA")],
        );
        let app = app(state);
        let request = SubmitOrdersRequest {
            player_id: "player-fra".into(),
            orders: vec![Order::Hold(HoldOrder {
                submitter: PowerId::from("FRA"),
                corps: CorpsId::from("CORPS_FRA_001"),
            })],
        };
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/games/game-a/orders")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let payload: SubmitOrdersResponse = body_json(response).await;
        assert_eq!(payload.accepted, 1);
        assert_eq!(payload.event_log_len, 0);
    }

    #[tokio::test]
    async fn get_events_empty_initially() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            Vec::new(),
            &[("player-fra", "FRA")],
        );
        let app = app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/game-a/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let payload: Vec<Event> = body_json(response).await;
        assert!(payload.is_empty());
    }

    #[tokio::test]
    async fn get_events_since_reconnect() {
        let state = test_state();
        let events = vec![
            Event::OrderRejected(OrderRejected {
                reason_code: "A".into(),
                message: "first".into(),
            }),
            Event::OrderRejected(OrderRejected {
                reason_code: "B".into(),
                message: "second".into(),
            }),
            Event::OrderRejected(OrderRejected {
                reason_code: "C".into(),
                message: "third".into(),
            }),
        ];
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            events,
            &[("player-fra", "FRA")],
        );
        let app = app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/game-a/events?since=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let payload: Vec<Event> = body_json(response).await;
        assert_eq!(payload.len(), 2);
    }

    #[tokio::test]
    async fn create_multiple_games_isolated() {
        let state = test_state();
        let app = app(state.clone());
        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/games")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);
        }
        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions.len(), 2);
        let ids = sessions.keys().cloned().collect::<Vec<_>>();
        assert_ne!(ids[0], ids[1]);
    }

    #[tokio::test]
    async fn projected_state_filtered_by_player() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            Vec::new(),
            &[("player-fra", "FRA"), ("player-aus", "AUS")],
        );
        let app = app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/game-a/state?player_id=player-fra")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let payload: Scenario = body_json(response).await;
        assert!(payload.corps.contains_key(&CorpsId::from("CORPS_FRA_001")));
        assert!(!payload.corps.contains_key(&CorpsId::from("CORPS_AUS_001")));
    }

    #[tokio::test]
    async fn submit_order_unknown_game_404() {
        let app = app(test_state());
        let request = SubmitOrdersRequest {
            player_id: "player-fra".into(),
            orders: vec![],
        };
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/games/missing/orders")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn websocket_upgrades() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            Vec::new(),
            &[("player-fra", "FRA")],
        );
        let (addr, handle) = spawn_test_server(state).await;
        let url = format!("ws://{addr}/games/game-a/ws");

        let (mut socket, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("connect websocket");
        let message = socket
            .next()
            .await
            .expect("connected frame")
            .expect("websocket message");
        assert_eq!(message.into_text().unwrap(), "{\"type\":\"connected\"}");

        handle.abort();
    }

    #[tokio::test]
    async fn get_state_unknown_player_404() {
        let state = test_state();
        insert_session(&state, "game-a", scenario_fixture(), Vec::new(), &[]);
        let app = app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/game-a/state?player_id=ghost")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn submit_orders_unknown_player_404() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            Vec::new(),
            &[("player-fra", "FRA")],
        );
        let app = app(state);
        let request = SubmitOrdersRequest {
            player_id: "ghost".into(),
            orders: vec![],
        };
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/games/game-a/orders")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn submit_orders_rejects_submitter_mismatch() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            Vec::new(),
            &[("player-fra", "FRA")],
        );
        let app = app(state);
        let request = SubmitOrdersRequest {
            player_id: "player-fra".into(),
            orders: vec![Order::Hold(HoldOrder {
                submitter: PowerId::from("AUS"),
                corps: CorpsId::from("CORPS_AUS_001"),
            })],
        };
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/games/game-a/orders")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_events_since_beyond_end_returns_empty() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            vec![Event::OrderRejected(OrderRejected {
                reason_code: "ONLY".into(),
                message: "one".into(),
            })],
            &[("player-fra", "FRA")],
        );
        let app = app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/game-a/events?since=99")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let payload: Vec<Event> = body_json(response).await;
        assert!(payload.is_empty());
    }

    #[tokio::test]
    async fn events_unknown_game_404() {
        let app = app(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/missing/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn projected_state_shows_enemy_corps_in_my_area() {
        let state = test_state();
        let mut scenario = scenario_fixture();
        scenario
            .corps
            .get_mut(&CorpsId::from("CORPS_AUS_001"))
            .unwrap()
            .area = AreaId::from("AREA_PARIS");
        insert_session(
            &state,
            "game-a",
            scenario,
            Vec::new(),
            &[("player-fra", "FRA")],
        );
        let app = app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/games/game-a/state?player_id=player-fra")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let payload: Scenario = body_json(response).await;
        assert!(payload.corps.contains_key(&CorpsId::from("CORPS_AUS_001")));
    }

    #[tokio::test]
    async fn order_submission_does_not_mutate_other_game() {
        let state = test_state();
        insert_session(
            &state,
            "game-a",
            scenario_fixture(),
            Vec::new(),
            &[("player-fra", "FRA")],
        );
        insert_session(
            &state,
            "game-b",
            scenario_fixture(),
            vec![Event::OrderRejected(OrderRejected {
                reason_code: "KEEP".into(),
                message: "keep".into(),
            })],
            &[("player-fra", "FRA")],
        );
        let app = app(state.clone());
        let request = SubmitOrdersRequest {
            player_id: "player-fra".into(),
            orders: vec![Order::Move(MoveOrder {
                submitter: PowerId::from("FRA"),
                corps: CorpsId::from("CORPS_FRA_001"),
                to: AreaId::from("AREA_PARIS"),
            })],
        };
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/games/game-a/orders")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let sessions = state.sessions.lock().unwrap();
        assert_eq!(sessions.get("game-b").unwrap().event_log.len(), 1);
    }

    #[tokio::test]
    async fn create_game_initializes_empty_event_log() {
        let state = test_state();
        let app = app(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/games")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let payload: CreateGameResponse = body_json(response).await;
        let sessions = state.sessions.lock().unwrap();
        assert!(sessions.get(&payload.game_id).unwrap().event_log.is_empty());
    }

    #[tokio::test]
    async fn websocket_unknown_game_404() {
        let (addr, handle) = spawn_test_server(test_state()).await;
        let url = format!("ws://{addr}/games/missing/ws");

        let err = tokio_tungstenite::connect_async(&url)
            .await
            .expect_err("missing game should reject websocket");
        match err {
            tokio_tungstenite::tungstenite::Error::Http(response) => {
                assert_eq!(response.status(), StatusCode::NOT_FOUND);
            }
            other => panic!("expected http error, got {other:?}"),
        }

        handle.abort();
    }
}
