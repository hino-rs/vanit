use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::{moderation::scoring_violates_terms, schema::ReportRequest, state::AppState};

const BLACKLIST_SCORE_THRESHOLD: f32 = 0.1;
const PENALTY_HOURS_PER_SCORE: f32 = 10.0;
const MIN_PENALTY_HOURS: i64 = 1;

pub async fn get_people_count(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (matched, waiting) = state.pair_manager.count_data();
    Json(json!({ "matched": matched, "waiting": waiting }))
}

pub async fn report(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReportRequest>,
) -> StatusCode {
    let score = scoring_violates_terms(&state.openai_client, &request.chat).await;
    // スコアが0.1以上で対象入り
    if score >= BLACKLIST_SCORE_THRESHOLD {
        let penalty = ((score * PENALTY_HOURS_PER_SCORE) as i64).max(MIN_PENALTY_HOURS); // 最低一時間
        state
            .pair_manager
            .add_to_blacklist(request.target_user_id, penalty)
            .await;
    }
    StatusCode::OK
}
