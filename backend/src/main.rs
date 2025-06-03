use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use warp::Filter;
use warp::fs::File;
use std::convert::Infallible;

mod game;
use game::{GameState, Move};
use serde_json;

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(100);
    let tx = Arc::new(Mutex::new(tx));
    let tx_ws = tx.clone();
    // Shared game state for all connections
    let game_state = Arc::new(Mutex::new(GameState::new()));
    let game_state_ws = game_state.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let tx = tx_ws.clone();
            let game_state = game_state_ws.clone();
            ws.on_upgrade(move |websocket| handle_connection(websocket, tx, game_state))
        });
    // Static file handler
    let static_route = warp::path::end()
        .and(warp::fs::file("static/index.html"));
    // Combine routes
    let routes = ws_route.or(static_route);
    println!("Server listening on 127.0.0.1:8080");
    // Start the server
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
async fn handle_connection(
    ws: warp::ws::WebSocket,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
    game_state: Arc<Mutex<GameState>>,
) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    
    let mut rx = tx.lock().unwrap().subscribe();
    // broadcast current game state once on new connection
    let init_state = {
        let gs = game_state.lock().unwrap();
        serde_json::to_string(&*gs).expect("Failed to serialize initial game state")
    };
    tx.lock().unwrap().send(init_state).expect("Failed to broadcast initial game state");
        
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(warp::ws::Message::text(msg)).await.is_err() {
                break;
            }
        }
    });
    // ignore incoming messages for now
    while let Some(_msg) = ws_receiver.next().await {
        
    }
}