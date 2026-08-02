use std::sync::atomic::{AtomicU64, Ordering};

use async_openai::{Client, config::OpenAIConfig};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use log::info;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::schema;

type Tx = mpsc::UnboundedSender<String>;

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("パートナーが見つかりません")]
    PartnerNotFound,
    #[error("送信に失敗しました")]
    SendFailed,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Language {
    Japanese,
    #[default]
    English,
    Irish, // ライルランド語
    Dutch, // オランダ語
    German,
    French,
    Italian,
    Spanish,
    Portuguese,
    Korean,
    Malay,
    Filipino,
    Indonesian,
    Thai,
}

impl Language {
    pub fn parse_or_default(str: &str) -> Language {
        match str {
            "ja" => Self::Japanese,
            "en" => Self::English,
            "ga" => Self::Irish,
            "nl" => Self::Dutch,
            "de" => Self::German,
            "fr" => Self::French,
            "it" => Self::Italian,
            "es" => Self::Spanish,
            "pt" => Self::Portuguese,
            "ko" => Self::Korean,
            "ms" => Self::Malay,
            "fil" => Self::Filipino,
            "id" => Self::Indonesian,
            "th" => Self::Thai,
            _ => Self::default(),
        }
    }
}

#[derive(Clone)]
pub struct User {
    pub tx: Tx,
    pub language: Language,
}

#[derive(Default)]
pub struct AppState {
    pub pair_manager: PairManager,
    pub openai_client: Client<OpenAIConfig>,
}
/// ペア管理システム
#[derive(Default)]
pub struct PairManager {
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
    pub fn register(&self, my_id: Uuid, my_data: User) -> Option<(Uuid, Tx)> {
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
            // 双方向にactive_pairsへ登録
            self.active_pairs
                .insert(my_id, (partner_id, partner_data.tx.clone()));
            self.active_pairs
                .insert(partner_id, (my_id, my_data.tx.clone()));

            return Some((partner_id, partner_data.tx));
        }

        // 待機者がいなければ自分が待合室に入る
        self.waiting.insert(my_id, my_data);
        info!("待機中: {}人", self.waiting.len());
        None
    }

    /// メッセージを相手に転送する
    pub fn send_to_partner(&self, my_id: &Uuid, message: String) -> Result<(), SendError> {
        let partner_tx = self
            .active_pairs
            .get(my_id)
            .map(|guard| guard.value().1.clone());

        if let Some(tx) = partner_tx {
            tx.send(message).map_err(|_| SendError::SendFailed)
        } else {
            Err(SendError::PartnerNotFound)
        }
    }

    /// 切断時のクリーンアップ
    pub fn unregister(&self, my_id: &Uuid) {
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
    pub fn count_data(&self) -> (u64, u64) {
        (
            self.matched_count.load(Ordering::Relaxed),
            self.waiting.len() as u64,
        )
    }

    /// ブラックリスト登録
    pub async fn add_to_blacklist(&self, device_id: Uuid, add_hours: i64) {
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
