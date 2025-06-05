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
use game::{GameState, legal_moves_for_piece_strict, Color, PieceType};
use serde_json;
use serde_json::json;

type Clients = Arc<TokioMutex<HashMap<usize, Arc<TokioMutex<SplitSink<WebSocket, WsMessage>>>>>>;
static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    // check for silent mode to suppress game logs
    let silent = std::env::args().any(|arg| arg == "--silent");
    // track connected clients
    let clients: Clients = Arc::new(TokioMutex::new(HashMap::new()));
    let (tx, _rx) = broadcast::channel(100);
    let tx = Arc::new(Mutex::new(tx));
    let tx_ws = tx.clone();
    // Shared game state for all connections
    let game_state = Arc::new(Mutex::new(GameState::new()));
    let game_state_ws = game_state.clone();
    let clients_ws = clients.clone();  
    let silent_ws = silent;
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || (clients_ws.clone(), tx_ws.clone(), game_state_ws.clone(), silent_ws)))
        .map(|ws: warp::ws::Ws, (clients, tx, game_state, silent)| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients, tx, game_state, silent))
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
        .run(([0, 0, 0, 0], 8080))
        .await;
}
async fn handle_connection(
    ws: warp::ws::WebSocket,
    clients: Clients,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
    game_state: Arc<Mutex<GameState>>,
    silent: bool,
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
    // determine and send role assignment
    let role_str = {
        let clients_guard = clients.lock().await;
        match clients_guard.len() {
            1 => "white",
            2 => "black",
            _ => "observer",
        }
    };
    let assign_msg = serde_json::to_string(&json!({
        "instruction_type": "assign_color",
        "color": role_str
    })).expect("Failed to serialize assign_color");
    {
        let mut sink = ws_tx.lock().await;
        let _ = sink.send(WsMessage::text(assign_msg)).await;
    }
    // send initial full game state to this client
    let init_state = {
        let gs = game_state.lock().unwrap();
        // print the current game code for debugging if not silent
        if !silent { println!("Game code: {}", gs.game_code()); }
        // serialize game state then add check/checkmate flags
        let mut val = serde_json::to_value(&*gs).expect("Serialize to Value");
        val["in_check"] = serde_json::Value::Bool(gs.is_in_check());
        val["is_checkmate"] = serde_json::Value::Bool(gs.is_checkmate());
        serde_json::to_string(&val).expect("Failed to serialize initial game state")
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
    // server-side role enforcement
    let my_role = role_str.to_string();
    // handle incoming messages
    while let Some(Ok(msg)) = ws_rx.next().await {
        if msg.is_text() {
            if let Ok(text) = msg.to_str() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                    // dispatch based on instruction type
                    match value.get("instruction_type").and_then(|v| v.as_str()) {
                        Some("get_legal_moves") => {
                            // observers cannot see legal moves
                            if my_role == "observer" { break; }
                            if let Some(s) = value.get("square_clicked").and_then(|v| v.as_str()) {
                                if let Ok(idx) = s.parse::<u8>() {
                                    // remember source for next move
                                    last_move_from = Some(idx);
                                    // computer legal move targets
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
                                        let mut gs = game_state.lock().unwrap();
                                        if (my_role == "white" && gs.piece_color_at(from as usize) == Some(Color::White))
                                          || (my_role == "black" && gs.piece_color_at(from as usize) == Some(Color::Black))
                                        {
                                            // parse optional promotion piece
                                            let promotion = value.get("promotion")
                                                .and_then(|v| v.as_str())
                                                .and_then(|s| match s {
                                                    "queen" => Some(PieceType::Queen),
                                                    "rook" => Some(PieceType::Rook),
                                                    "bishop" => Some(PieceType::Bishop),
                                                    "knight" => Some(PieceType::Knight),
                                                    _ => None,
                                                });
                                            // logging context
                                            let piece_color_enum = gs.piece_color_at(from as usize).unwrap();
                                            let piece_type_enum = gs.piece_type_at(from as usize).unwrap();
                                            let from_file = (b'a' + (from % 8)) as char;
                                            let from_rank = (from / 8 + 1).to_string();
                                            let from_coord = format!("{}{}", from_file.to_ascii_uppercase(), from_rank);
                                            let to_file = (b'a' + (dest % 8)) as char;
                                            let to_rank = (dest / 8 + 1).to_string();
                                            let to_coord = format!("{}{}", to_file.to_ascii_uppercase(), to_rank);
                                            // log events if not silent
                                            if !silent {
                                                println!("{:?} {:?} moved from {} to {}", piece_color_enum, piece_type_enum, from_coord, to_coord);
                                                // detect and log capture
                                                let captured_color_opt = gs.piece_color_at(dest as usize);
                                                let captured_type_opt = gs.piece_type_at(dest as usize);
                                                if let (Some(captured_color), Some(captured_type)) = (captured_color_opt, captured_type_opt) {
                                                    println!("{:?} {:?} captured by {:?} {:?}", captured_color, captured_type, piece_color_enum, piece_type_enum);
                                                }
                                                // check and checkmate
                                                if gs.is_in_check() {
                                                    println!("{:?} in Check", gs.turn());
                                                }
                                                if gs.is_checkmate() {
                                                    println!("{:?} in Checkmate", gs.turn());
                                                }
                                            }
                                            // apply the move
                                            gs.move_piece(from, dest, promotion);
                                            // broadcast updated full state
                                            // serialize updated state with check/checkmate
                                            let mut val = serde_json::to_value(&*gs).expect("Serialize to Value");
                                            val["in_check"] = serde_json::Value::Bool(gs.is_in_check());
                                            val["is_checkmate"] = serde_json::Value::Bool(gs.is_checkmate());
                                            let full = serde_json::to_string(&val)
                                                .expect("Failed to serialize game state");
                                            tx.lock().unwrap().send(full).unwrap();
                                            last_move_from = None;
                                        } else {
                                            eprintln!("Illegal move by {} on {}", my_role, from);
                                        }
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

