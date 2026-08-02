use async_openai::{
    Client,
    config::OpenAIConfig,
    types::moderations::{CreateModerationRequestArgs, ModerationInput},
};

const GENERAL_SCORE_NOISE_FLOOR: f32 = 0.05;

pub async fn scoring_violates_terms(openai_client: &Client<OpenAIConfig>, chat: &[String]) -> f32 {
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

        let effective_general = if general_score > GENERAL_SCORE_NOISE_FLOOR {
            general_score
        } else {
            0.0
        };

        return critical_score.max(effective_general);
    }

    0.0
}
