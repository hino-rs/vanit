use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// タグ付きメッセージ種別
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Chat { content: String },
    System { event: SystemEvent, message: String },
}

/// システムイベントの種類
#[derive(Debug, Clone, Deserialize, PartialEq, TS, Serialize)]
#[ts(export, export_to = "../../frontend/src/lib/types/")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemEvent {
    PartnerDisconnected,
    MatchingCompleted { partner_id: Uuid },
    FailedToSendMessage,
}

/// 通報リクエスト
#[derive(Debug, Clone, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/")]
pub struct ReportRequest {
    pub target_user_id: Uuid,
    pub reason: ReportReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// 通報理由の選択肢
#[derive(Debug, Clone, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/")]
#[serde(rename_all = "snake_case")]
pub enum ReportReason {
    /// 不適切な発言
    InappropriateLanguage,
    /// スパム
    Spam,
    /// 嫌がらせ
    Harassment,
    /// その他
    Other,
}

impl ReportReason {
    pub fn penalty(&self) -> i64 {
        match self {
            ReportReason::InappropriateLanguage => 4,
            ReportReason::Spam => 12,
            ReportReason::Harassment => 8,
            ReportReason::Other => 2,
        }
    }
}
