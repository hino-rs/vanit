use serde::Deserialize;
use ts_rs::TS;
use uuid::Uuid;

/// タグ付きメッセージ種別
#[derive(Debug, Clone, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Chat { content: String },
    System { event: SystemEvent, message: String },
}

/// システムイベントの種類
#[derive(Debug, Clone, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemEvent {
    PartnerDisconnected,
    MatchingCompleted,
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
    InappropriateLanguage,
    Spam,
    Harassment,
    Other,
}
