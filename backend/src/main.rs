use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, mpsc, oneshot};
use tower_http::cors::CorsLayer;

/// 待機中のユーザー
type PeerInfo = (
    mpsc::UnboundedSender<String>,
    oneshot::Sender<mpsc::UnboundedSender<String>>,
);

#[derive(Default)]
struct AppState {
    waiting_peer: Mutex<Option<PeerInfo>>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cors = CorsLayer::permissive();
    let state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    // 自分の受信ボックスを作る
    let (my_tx, mut my_rx) = mpsc::unbounded_channel::<String>();
    let partner_tx: mpsc::UnboundedSender<String>;

    // === ペアリング処理 ===
    // 待合室を確認
    let mut lock = state.waiting_peer.lock().await;
    if let Some((waiting_tx, oneshot_tx)) = lock.take() {
        // --- 既に誰かが待機していた場合 ---
        // 相手の宛先を自分のpartner_txにセット
        partner_tx = waiting_tx;
        // 相手に自分の宛先を教える
        let _ = oneshot_tx.send(my_tx);

        drop(lock);
    } else {
        // --- 誰も待っていなかった場合 ---
        // 1回限りの通信チャネルを作る
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        // 自分の宛先と返事をもらうためのチャネルを待合室に置いておく
        *lock = Some((my_tx, oneshot_tx));
        drop(lock);

        // 誰かが来て、自分に宛先を教えてくれるまで待つ
        match oneshot_rx.await {
            // 相手が来たらもらった宛先をpartner_txにセット
            Ok(tx) => partner_tx = tx,
            Err(_) => return,
        }
    }

    // ペアリング出来たらクライアントに完了を伝える
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let _ = ws_sender
        .send(Message::Text("ペアリングが完了しました！".into()))
        .await;

    // === メッセージの送受信ループ ===
    loop {
        tokio::select! {
            // パートナーからメッセージが届いたとき
            some_msg = my_rx.recv() => {
                match some_msg {
                    Some(msg) => {
                        // 自分のWebSocketへ送信
                        if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        // パートナー側の通信チャネルが切断された場合
                        let _ = ws_sender.send(Message::Text("パートナーが切断しました。".into())).await;
                        break;
                    }
                }
            }
            // 自分のWebSocketからメッセージが送られてきたとき
            ws_msg = ws_receiver.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        // パートナーの宛先へ送信
                        if partner_tx.send(text.to_string()).is_err() {
                            let _ = ws_sender.send(Message::Text("パートナーへの送信に失敗しました。".into())).await;
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        // 自分が切断した
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ping/Pong/Binaryなど
                    }
                    Some(Err(_)) => {
                        break;
                    }
                }
            }
        }
    }
}
