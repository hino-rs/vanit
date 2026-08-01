mod schema;
use chrono::{DateTime, Duration, Utc};
use schema::*;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::moderations::{CreateModerationRequestArgs, ModerationInput},
};
use axum::{
    Json, Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use log::info;
use serde::Deserialize;
use serde_json::json;
use std::{
    net::SocketAddr,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

type Tx = mpsc::UnboundedSender<String>;

async fn scoring_violates_terms(openai_client: &Client<OpenAIConfig>, chat: &[String]) -> f32 {
    let combined_chat = chat.join("\n");
    if combined_chat.trim().is_empty() {
        return 0.0;
    }

    let request = match CreateModerationRequestArgs::default()
        .model("omni-moderation-latest")
        .input(ModerationInput::String(combined_chat))
        .build()
    {
        Ok(req) => req,
        Err(_) => return 0.0,
    };

    let response = match openai_client.moderations().create(request).await {
        Ok(res) => res,
        Err(err) => {
            log::error!("OpenAI APIエラー: {:?}", err);
            return 0.0;
        }
    };

    if let Some(result) = response.results.first() {
        let scores = &result.category_scores;

        // 重大リスク（即時・重度BAN対象）の最高値
        let critical_score = [
            scores.sexual_minors,
            scores.self_harm_instructions,
            scores.hate_threatening,
            scores.harassment_threatening,
            scores.illicit_violent,
        ]
        .into_iter()
        .fold(0.0f32, f32::max);

        // 一般的な不適切な内容の最高値
        let general_score = [
            scores.hate,
            scores.harassment,
            scores.sexual,
            scores.violence,
            scores.self_harm,
            scores.illicit,
        ]
        .into_iter()
        .fold(0.0f32, f32::max);

        let effective_general = if general_score > 0.05 {
            general_score
        } else {
            0.0
        };

        return critical_score.max(effective_general);
    }

    0.0
}

#[derive(Deserialize)]
struct ConnectQuery {
    user_id: Uuid,
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
    openai_client: Client<OpenAIConfig>,
}
/// ペア管理システム
#[derive(Default)]
struct PairManager {
    /// 待機中のユーザー
    waiting: DashMap<Uuid, User>,
    /// ペアリング済みのユーザー
    active_pairs: DashMap<Uuid, (Uuid, Tx)>,
    /// 接続済みユーザー数
    matched_count: AtomicU64,
    /// ブラックリスト ID, 解除までの残り時間
    blacklist: DashMap<Uuid, DateTime<Utc>>,
}

impl PairManager {
    /// ユーザーが参加したときの処理
    fn register(&self, my_id: Uuid, my_data: User) -> Option<(Uuid, Tx)> {
        // 待機中のIDを1つ取得 (スコープを区切ってReadLockを即座に解放する)
        let partner_id = {
            self.waiting
                .iter()
                .find(|user| user.language == my_data.language)
                .map(|user| *user.key())
        };

        // 待合室から相手を削除
        if let Some(partner_id) = partner_id
            && let Some((_, partner_data)) = self.waiting.remove(&partner_id)
        {
            self.matched_count.fetch_add(2, Ordering::Relaxed);
            if partner_data.language == my_data.language {
                // 双方向にactive_pairsへ登録
                self.active_pairs
                    .insert(my_id, (partner_id, partner_data.tx.clone()));
                self.active_pairs
                    .insert(partner_id, (my_id, my_data.tx.clone()));

                return Some((partner_id, partner_data.tx));
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
            self.matched_count.fetch_sub(2, Ordering::Relaxed);
            // 相手側のペアリング情報も削除する
            self.active_pairs.remove(&partner_id);
            // 相手に切断メッセージを通知
            let msg = schema::Message::System {
                event: schema::SystemEvent::PartnerDisconnected,
                message: "パートナーが切断しました".into(),
            };
            let _ = partner_tx.send(serde_json::to_string(&msg).unwrap());
        }
    }

    /// 待機中の人数を返す(マッチング者数, 待機者数)
    async fn count_data(&self) -> (u64, u64) {
        (
            self.matched_count.load(Ordering::Relaxed),
            self.waiting.len() as u64,
        )
    }

    /// ブラックリスト登録
    async fn add_to_blacklist(&self, device_id: Uuid, add_hours: i64) {
        let now = Utc::now();
        self.blacklist
            .entry(device_id)
            .and_modify(|until| {
                // 既存のバン期間が残っていればそこから延長、切れていれば現在時刻から加算
                let base_time = if *until > now { *until } else { now };
                *until = base_time + Duration::hours(add_hours);
            })
            .or_insert_with(|| now + Duration::hours(add_hours));
    }

    /// バン状態の確認
    pub fn is_blacklisted(&self, device_id: &Uuid) -> bool {
        if let Some(until) = self.blacklist.get(device_id)
            && Utc::now() < *until.value()
        {
            return true;
        }

        // 期限切れの場合はブラックリストから削除
        self.blacklist.remove(device_id);
        false
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cors = CorsLayer::permissive();

    let mut app_state = AppState::default();

    if let Ok(path) = std::env::current_dir() {
        println!("カレントディレクトリ: {path:?}");
    }
    match dotenvy::dotenv() {
        Ok(path) => println!(".env を読み込みました: {:?}", path),
        Err(e) => println!(".env の読み込みに失敗しました: {:?}", e),
    }

    match std::env::var("OPENAI_API_KEY") {
        Ok(val) => println!("OPENAI_API_KEY 取得成功 (文字数: {})", val.len()),
        Err(e) => println!("OPENAI_API_KEY 取得失敗: {:?}", e),
    }

    let openai_client = Client::new();

    app_state.openai_client = openai_client;

    let state = Arc::new(app_state);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/get_people_count", get(get_people_count))
        .route("/api/report", post(report))
        .route("/api/blacklisted_check", post(blacklisted_check))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_people_count(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (matched, waiting) = state.pair_manager.count_data().await;
    Json(json!({ "matched": matched, "waiting": waiting }))
}

async fn report(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReportRequest>,
) -> StatusCode {
    let score = scoring_violates_terms(&state.openai_client, &request.chat).await;
    // スコアが0.1以上で対象入り
    if score >= 0.1 {
        let penalty = ((score * 10.0) as i64).max(1); // 最低一時間
        state
            .pair_manager
            .add_to_blacklist(request.target_user_id, penalty)
            .await;
        return StatusCode::OK;
    }
    StatusCode::OK
}

async fn blacklisted_check(State(state): State<Arc<AppState>>, device_id: String) {
    if let Ok(uuid) = Uuid::from_str(&device_id) {
        // 一旦バンかは知らせない
        let _ = state.pair_manager.is_blacklisted(&uuid);
    }
}

async fn ws_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ConnectQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    let is_blacklisted = state.pair_manager.blacklist.contains_key(&query.user_id);

    Ok(ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            state,
            query.user_id,
            Language::from_str(&query.lang),
            is_blacklisted,
        )
    }))
}

async fn handle_socket(
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
