use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::state::AppState;
use crate::{
    schema,
    state::{Language, User},
};

#[derive(Deserialize)]
pub struct ConnectQuery {
    user_id: Uuid,
    lang: String,
}

pub async fn ws_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ConnectQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    let is_blacklisted = state.pair_manager.is_blacklisted(&query.user_id);

    Ok(ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            state,
            query.user_id,
            Language::parse_or_default(&query.lang),
            is_blacklisted,
        )
    }))
}

pub async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    my_id: Uuid,
    language: Language,
    is_blacklisted: bool,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    // 自分の受信ボックスを作る
    let (my_tx, mut my_rx) = mpsc::unbounded_channel::<String>();

    // ブラックリスト対象者をループに閉じ込める
    if is_blacklisted {
        loop {
            tokio::select! {
                ws_msg = ws_receiver.next() => {
                    match ws_msg {
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
                _ = my_rx.recv() => {}
            }
        }
        return;
    }

    let user = User {
        tx: my_tx,
        language,
    };

    // === ペアリング処理 ===
    if let Some((partner_id, partner_tx)) = state.pair_manager.register(my_id, user) {
        let msg_for_me = schema::Message::System {
            event: schema::SystemEvent::MatchingCompleted { partner_id },
            message: "ペアリングが完了しました！".into(),
        };

        let msg_for_partner = schema::Message::System {
            event: schema::SystemEvent::MatchingCompleted { partner_id: my_id },
            message: "ペアリングが完了しました".into(),
        };

        // 後から参加した側がマッチングを完成させた場合
        let _ = ws_sender
            .send(Message::Text(
                serde_json::to_string(&msg_for_me).unwrap().into(),
            ))
            .await;

        // 待機中だった相手にもペアリング完了を通知
        let _ = partner_tx.send(serde_json::to_string(&msg_for_partner).unwrap());
    }

    // === メッセージの送受信ループ ===
    loop {
        tokio::select! {
            // パートナーまたはシステムからメッセージが届いたとき
            some_data = my_rx.recv() => {
                match some_data {
                    Some(data) => {
                        // 自分のWebSocketへ送信
                        if ws_sender.send(Message::Text(data.into())).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        // 通信チャネルが切断された場合
                        break;
                    }
                }
            }
            // 自分のWebSocketからメッセージが送られてきたとき
            ws_msg = ws_receiver.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        // パートナーの宛先へ送信
                        if state.pair_manager.send_to_partner(&my_id, text.to_string()).is_err() {
                            let msg = schema::Message::System {
                                event: schema::SystemEvent::FailedToSendMessage,
                                message: "メッセージの送信に失敗しました".into(),
                            };
                            let _ = ws_sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await;
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

    // === クリーンアップ ===
    state.pair_manager.unregister(&my_id);
}
