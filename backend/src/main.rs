use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use log::info;
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

type Tx = mpsc::UnboundedSender<String>;

#[derive(Deserialize)]
struct ConnectQuery {
    lang: String,
}

#[derive(Clone, Default, Debug, PartialEq)]
enum Language {
    Japanese,
    #[default]
    English,
    Chinese,
    Hindi,
    Spanish,
    Arabic,
    French,
    Bengali,
    Portuguese,
    Indonesian,
    Urdu,
    Russian,
    German,
    NigerianPidgin,
    EgyptianArabic,
}

impl Language {
    fn from_str(str: &str) -> Language {
        match str {
            "ja" => Self::Japanese,
            "en" => Self::English,
            "zh" => Self::Chinese,
            "hi" => Self::Hindi,
            "es" => Self::Spanish,
            "ar" => Self::Arabic,
            "fr" => Self::French,
            "bn" => Self::Bengali,
            "pt" => Self::Portuguese,
            "id" => Self::Indonesian,
            "ur" => Self::Urdu,
            "ru" => Self::Russian,
            "de" => Self::German,
            "pcm" => Self::NigerianPidgin,
            "arz" => Self::EgyptianArabic,
            _ => Self::default(),
        }
    }
}

#[derive(Clone)]
struct User {
    tx: Tx,
    language: Language,
}

#[derive(Default)]
struct AppState {
    pair_manager: PairManager,
}

/// ペア管理システム
#[derive(Default)]
struct PairManager {
    /// 待機中のユーザー
    waiting: DashMap<Uuid, User>,
    /// ペアリング済みのユーザー
    active_pairs: DashMap<Uuid, (Uuid, Tx)>,
}

impl PairManager {
    /// ユーザーが参加したときの処理
    fn register(&self, my_id: Uuid, my_data: User) -> Option<(Uuid, Tx)> {
        // 待機中のIDを1つ取得 (スコープを区切ってReadLockを即座に解放する)
        let partner_id = {
            if let Some(user) = self
                .waiting
                .iter()
                .find(|user| user.language == my_data.language)
            {
                Some(*user.key())
            } else {
                None
            }
        };

        // 待合室から相手を削除
        if let Some(partner_id) = partner_id {
            if let Some((_, partner_data)) = self.waiting.remove(&partner_id) {
                if partner_data.language == my_data.language {
                    info!("ペア成立: {:?}", partner_data.language);
                    // 双方向にactive_pairsへ登録
                    self.active_pairs
                        .insert(my_id, (partner_id, partner_data.tx.clone()));
                    self.active_pairs
                        .insert(partner_id, (my_id, my_data.tx.clone()));

                    return Some((partner_id, partner_data.tx));
                }
            }
        }

        // 待機者がいなければ自分が待合室に入る
        self.waiting.insert(my_id, my_data);
        info!("待機中: {}人", self.waiting.len());
        None
    }

    /// メッセージを相手に転送する
    fn send_to_partner(&self, my_id: &Uuid, message: String) -> Result<(), &'static str> {
        let partner_tx = self
            .active_pairs
            .get(my_id)
            .map(|guard| guard.value().1.clone());

        if let Some(tx) = partner_tx {
            tx.send(message).map_err(|_| "送信失敗")
        } else {
            Err("パートナーが見つかりません")
        }
    }

    /// 切断時のクリーンアップ
    fn unregister(&self, my_id: &Uuid) {
        // 待合室にいれば削除
        self.waiting.remove(my_id);

        // アクティブペアにいれば自分と相手のペアリングを解除
        if let Some((_, (partner_id, partner_tx))) = self.active_pairs.remove(my_id) {
            // 相手側のペアリング情報も削除する
            self.active_pairs.remove(&partner_id);
            // 相手に切断メッセージを通知
            let _ = partner_tx.send("パートナーが切断しました。".into());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
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

async fn ws_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ConnectQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, Language::from_str(&query.lang)))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, language: Language) {
    let my_id = Uuid::new_v4();
    // 自分の受信ボックスを作る
    let (my_tx, mut my_rx) = mpsc::unbounded_channel::<String>();

    let user = User {
        tx: my_tx,
        language,
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // === ペアリング処理 ===
    if let Some((_partner_id, partner_tx)) = state.pair_manager.register(my_id, user) {
        // 後から参加した側がマッチングを完成させた場合
        let _ = ws_sender
            .send(Message::Text("ペアリングが完了しました！".into()))
            .await;
        // 待機中だった相手にもペアリング完了を通知
        let _ = partner_tx.send("ペアリングが完了しました！".into());
    }

    // === メッセージの送受信ループ ===
    loop {
        tokio::select! {
            // パートナーまたはシステムからメッセージが届いたとき
            some_msg = my_rx.recv() => {
                match some_msg {
                    Some(msg) => {
                        // 自分のWebSocketへ送信
                        if ws_sender.send(Message::Text(msg.into())).await.is_err() {
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

    // === クリーンアップ ===
    state.pair_manager.unregister(&my_id);
}
