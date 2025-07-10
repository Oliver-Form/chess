use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex as TokioMutex};
use warp::Filter;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::stream::SplitSink;
use warp::ws::{Message as WsMessage, WebSocket};
use std::env;

mod game;
use game::{GameState, legal_moves_for_piece_strict, Color, PieceType};
use serde_json;
use serde_json::json;

type Clients = Arc<TokioMutex<HashMap<usize, Arc<TokioMutex<SplitSink<WebSocket, WsMessage>>>>>>;
static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
static GAME_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

struct GameRoom {
    game_state: Arc<TokioMutex<GameState>>,
    tx: broadcast::Sender<String>,
    clients: HashMap<usize, Arc<TokioMutex<SplitSink<WebSocket, WsMessage>>>>,
}

type GameRooms = Arc<TokioMutex<HashMap<usize, GameRoom>>>;

#[tokio::main]
async fn main() {
    // parse command-line flags
    let args: Vec<String> = env::args().collect();
    let silent = args.iter().any(|arg| arg == "--silent");
    let verbose = args.iter().any(|arg| arg == "--verbose");
    let silent_ws = silent;
    let verbose_ws = verbose;
    // track multiple games
    let game_rooms: GameRooms = Arc::new(TokioMutex::new(HashMap::new()));

    let game_rooms_ws = game_rooms.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || (game_rooms_ws.clone(), silent_ws, verbose_ws)))
        .map(|ws: warp::ws::Ws, (game_rooms, silent, verbose)| {
            ws.on_upgrade(move |socket| handle_connection(socket, game_rooms, silent, verbose))
        });
    // Static file handler for frontend
    let static_route = warp::path::end()
        .and(warp::fs::file("../frontend/index.html"));
    // Serve frontend assets (images, JS, CSS)
    let assets_route = warp::path("frontend")
        .and(warp::fs::dir("../frontend"));
    // Combine routes: WebSocket, index.html, and assets
    let routes = ws_route.or(static_route).or(assets_route);
    // Determine port from env or default to 8080
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    println!("Server listening on 0.0.0.0:{}", port);
    // Start the server
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}
async fn handle_connection(
    ws: WebSocket,
    game_rooms: GameRooms,
    silent: bool,
    verbose: bool,
) {
    // split into sink & stream, then store sink for later per-client pushes
    let (ws_tx, mut ws_rx) = ws.split();
    let ws_tx = Arc::new(TokioMutex::new(ws_tx));

    let client_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    // determine or create a game room and remember its ID
    let my_game_id: usize = {
        let mut rooms = game_rooms.lock().await;
        // pick existing room or create new
        let game_id = if let Some((&id, _)) = rooms.iter().find(|(_, r)| r.clients.len() < 2) {
            id
        } else {
            let new_id = GAME_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
            let new_state = Arc::new(TokioMutex::new(GameState::new()));
            let (tx, _rx) = broadcast::channel::<String>(100);
            rooms.insert(new_id, GameRoom { game_state: new_state.clone(), tx, clients: HashMap::new() });
            new_id
        };
        // register this client
        let room = rooms.get_mut(&game_id).unwrap();
        room.clients.insert(client_id, ws_tx.clone());
        // send initial state
        let gs_arc = room.game_state.clone();
        let init = {
            let gs = gs_arc.lock().await;
            if verbose || !silent { println!("Game code: {}", gs.game_code()); }
            let mut val = serde_json::to_value(&*gs).unwrap();
            val["in_check"] = serde_json::Value::Bool(gs.is_in_check());
            val["is_checkmate"] = serde_json::Value::Bool(gs.is_checkmate());
            val["is_stalemate"] = serde_json::Value::Bool(gs.is_stalemate());
            val["is_threefold_repetition"] = serde_json::Value::Bool(gs.is_threefold_repetition());
            val["is_fifty_move_draw"] = serde_json::Value::Bool(gs.is_fifty_move_draw());
            val["is_insufficient_material"] = serde_json::Value::Bool(gs.is_insufficient_material());
            serde_json::to_string(&val).unwrap()
        };
        // send initial state directly to this client so it renders immediately
        {
            let mut sink = ws_tx.lock().await;
            let _ = sink.send(WsMessage::text(init.clone())).await;
        }
        // also broadcast to other subscribers (e.g., opponent)
        let _ = room.tx.send(init);
        game_id
    };
    // subscribe to this game room's broadcast channel
    let mut rx = {
        let rooms = game_rooms.lock().await;
        rooms.get(&my_game_id).unwrap().tx.subscribe()
    };
    // determine and send role assignment (only white for first, black for second)
    let role_str = {
        let rooms = game_rooms.lock().await;
        let count = rooms.get(&my_game_id).unwrap().clients.len();
        if count == 1 { "white" } else { "black" }
    };
    let assign_msg = serde_json::to_string(&json!({
        "instruction_type": "assign_color",
        "color": role_str
    })).expect("Failed to serialize assign_color");
    {
        let mut sink = ws_tx.lock().await;
        let _ = sink.send(WsMessage::text(assign_msg)).await;
    }
        
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
                if verbose { println!("Received instruction from client: {}", text); }
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                    // dispatch based on instruction type
                    match value.get("instruction_type").and_then(|v| v.as_str()) {
                        Some("get_legal_moves") => {
                             if let Some(s) = value.get("square_clicked").and_then(|v| v.as_str()) {
                                 if let Ok(idx) = s.parse::<u8>() {
                                    // remember source for next move
                                    last_move_from = Some(idx);
                                    // computer legal move targets
                                    let positions = {
                                        let gs_arc = {
                                            let rooms = game_rooms.lock().await;
                                            rooms.get(&my_game_id).unwrap().game_state.clone()
                                        };
                                        let gs = gs_arc.lock().await;
                                        legal_moves_for_piece_strict(&gs, idx)
                                    };
                                    let json = serde_json::to_string(&positions).unwrap();
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
                                        // clone game state Arc and lock
                                        let gs_arc = {
                                            let rooms = game_rooms.lock().await;
                                            rooms.get(&my_game_id).unwrap().game_state.clone()
                                        };
                                        let mut gs = gs_arc.lock().await;
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
                                            // log events if in verbose mode or not silent
                                            if verbose || !silent {
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
                                            let mut val = serde_json::to_value(&*gs).unwrap();
                                            val["in_check"] = serde_json::Value::Bool(gs.is_in_check());
                                            val["is_checkmate"] = serde_json::Value::Bool(gs.is_checkmate());
                                            val["is_stalemate"] = serde_json::Value::Bool(gs.is_stalemate());
                                            val["is_threefold_repetition"] = serde_json::Value::Bool(gs.is_threefold_repetition());
                                            val["is_fifty_move_draw"] = serde_json::Value::Bool(gs.is_fifty_move_draw());
                                            val["is_insufficient_material"] = serde_json::Value::Bool(gs.is_insufficient_material());
                                            let full = serde_json::to_string(&val).unwrap();
                                            let rooms = game_rooms.lock().await;
                                            rooms.get(&my_game_id).unwrap().tx.send(full).unwrap();
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
                        Some("rematch") => {
                            // reset game state for this room
                            let mut rooms = game_rooms.lock().await;
                            if let Some(room) = rooms.get_mut(&my_game_id) {
                                // replace with new state
                                *room.game_state.lock().await = GameState::new();
                                // broadcast refreshed initial state
                                let gs = room.game_state.lock().await;
                                let mut val = serde_json::to_value(&*gs).unwrap();
                                val["in_check"] = serde_json::Value::Bool(gs.is_in_check());
                                val["is_checkmate"] = serde_json::Value::Bool(gs.is_checkmate());
                                val["is_stalemate"] = serde_json::Value::Bool(gs.is_stalemate());
                                val["is_threefold_repetition"] = serde_json::Value::Bool(gs.is_threefold_repetition());
                                val["is_fifty_move_draw"] = serde_json::Value::Bool(gs.is_fifty_move_draw());
                                val["is_insufficient_material"] = serde_json::Value::Bool(gs.is_insufficient_material());
                                let full = serde_json::to_string(&val).unwrap();
                                let _ = room.tx.send(full);
                                // clear pending move
                                last_move_from = None;
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
        let mut rooms = game_rooms.lock().await;
        if let Some(room) = rooms.get_mut(&my_game_id) {
            room.clients.remove(&client_id);
        }
    }
}

// 