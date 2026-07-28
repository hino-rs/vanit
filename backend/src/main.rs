use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(cors);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())       
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            println!("Received: {text}");
            if socket.send(Message::Text(format!("Echo: {text}").into())).await.is_err() {
                break;
            }
        }
    }
}