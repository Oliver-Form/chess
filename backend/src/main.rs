use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, Mutex as TokioMutex};
use warp::Filter;
use warp::fs::File;
use std::convert::Infallible;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::stream::SplitSink;
use warp::ws::{Message as WsMessage, WebSocket};

mod game;
use game::{GameState, legal_moves_for_piece_strict};
use serde_json;

type Clients = Arc<TokioMutex<HashMap<usize, Arc<TokioMutex<SplitSink<WebSocket, WsMessage>>>>>>;
static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    // track connected clients
    let clients: Clients = Arc::new(TokioMutex::new(HashMap::new()));
    let (tx, _rx) = broadcast::channel(100);
    let tx = Arc::new(Mutex::new(tx));
    let tx_ws = tx.clone();
    // Shared game state for all connections
    let game_state = Arc::new(Mutex::new(GameState::new()));
    let game_state_ws = game_state.clone();
    let clients_ws = clients.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || (clients_ws.clone(), tx_ws.clone(), game_state_ws.clone())))
        .map(|ws: warp::ws::Ws, (clients, tx, game_state)| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients, tx, game_state))
        });
    // Static file handler for frontend
    let static_route = warp::path::end()
        .and(warp::fs::file("../frontend/index.html"));
    // Serve frontend assets (images, JS, CSS)
    let assets_route = warp::path("frontend")
        .and(warp::fs::dir("../frontend"));
    // Combine routes: WebSocket, index.html, and assets
    let routes = ws_route.or(static_route).or(assets_route);
    println!("Server listening on 127.0.0.1:8080");
    // Start the server
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
async fn handle_connection(
    ws: warp::ws::WebSocket,
    clients: Clients,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
    game_state: Arc<Mutex<GameState>>,
) {
    // split into sink & stream, then store sink for later per-client pushes
    let (ws_tx, mut ws_rx) = ws.split();
    let ws_tx = Arc::new(TokioMutex::new(ws_tx));

    let client_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    // register this client's sink
    {
        let mut clients_guard = clients.lock().await;
        clients_guard.insert(client_id, ws_tx.clone());
    }
    let mut rx = tx.lock().unwrap().subscribe();
    // send initial full game state to this client
    let init_state = {
        let gs = game_state.lock().unwrap();
        // print the current game code for debugging
        println!("Game code: {}", gs.game_code());
        serde_json::to_string(&*gs).expect("Failed to serialize initial game state")
    };
    tx.lock().unwrap().send(init_state).expect("Failed to broadcast initial game state");
        
    // clone sink for background game state broadcasts
    let broadcast_tx = ws_tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut sink = broadcast_tx.lock().await;
            let _ = sink.send(WsMessage::text(msg)).await;
        }
    });
    // state tracking for pending move
    let mut last_move_from: Option<u8> = None;
    // handle incoming messages
    while let Some(Ok(msg)) = ws_rx.next().await {
        if msg.is_text() {
            if let Ok(text) = msg.to_str() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                    // dispatch based on instruction type
                    match value.get("instruction_type").and_then(|v| v.as_str()) {
                        Some("get_legal_moves") => {
                            if let Some(s) = value.get("square_clicked").and_then(|v| v.as_str()) {
                                if let Ok(idx) = s.parse::<u8>() {
                                    // remember source for next move
                                    last_move_from = Some(idx);
                                    // compute legal move targets
                                    let positions = {
                                        let gs = game_state.lock().unwrap();
                                        legal_moves_for_piece_strict(&gs, idx)
                                    };
                                    let json = serde_json::to_string(&positions)
                                        .expect("Failed to serialize positions");
                                    // send highlights only to this client
                                    let mut sink = ws_tx.lock().await;
                                    let _ = sink.send(WsMessage::text(json)).await;
                                }
                            }
                        }
                        Some("request_move") => {
                            if let Some(dest_s) = value.get("destination").and_then(|v| v.as_str()) {
                                if let Ok(dest) = dest_s.parse::<u8>() {
                                    if let Some(from) = last_move_from {
                                        // apply the move on game state
                                        {
                                            let mut gs = game_state.lock().unwrap();
                                            gs.move_piece(from, dest);
                                        }
                                        // broadcast updated full state
                                        let full = {
                                            let gs = game_state.lock().unwrap();
                                            serde_json::to_string(&*gs)
                                                .expect("Failed to serialize game state")
                                        };
                                        tx.lock().unwrap()
                                            .send(full)
                                            .expect("Failed to broadcast updated state");
                                        // clear pending
                                        last_move_from = None;
                                    } else {
                                        eprintln!("No source square for move");
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    // unregister client on disconnect
    {
        let mut clients_guard = clients.lock().await;
        clients_guard.remove(&client_id);
    }
}
