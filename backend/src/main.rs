mod handlers;
mod moderation;
mod schema;
mod state;

use async_openai::Client;
use axum::{
    Router,
    routing::{get, post},
};
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

use crate::handlers::api::*;
use crate::handlers::ws::*;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cors = CorsLayer::permissive();

    if let Ok(path) = std::env::current_dir() {
        info!("カレントディレクトリ: {path:?}");
    }
    match dotenvy::dotenv() {
        Ok(path) => info!(".env を読み込みました: {:?}", path),
        Err(e) => info!(".env の読み込みに失敗しました: {:?}", e),
    }

    match std::env::var("OPENAI_API_KEY") {
        Ok(val) => info!("OPENAI_API_KEY 取得成功 (文字数: {})", val.len()),
        Err(e) => info!("OPENAI_API_KEY 取得失敗: {:?}", e),
    }

    let database_url = dotenvy::from_path_iter(".env")
        .ok()
        .and_then(|iter| {
            iter.filter_map(Result::ok)
                .find(|(key, _)| key == "DATABASE_URL")
                .map(|(_, value)| value)
        })
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL が設定されていません"))?;
    // sqlxがchannel_bindingパラメータを認識せず接続がハングするため、
    // Neon が付与するこのクエリパラメータを取り除いてから接続する。
    let database_url = database_url
        .replace("channel_binding=require&", "")
        .replace("&channel_binding=require", "")
        .replace("?channel_binding=require", "?");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    info!("Neon (Postgres) への接続に成功しました");

    let openai_client = Client::new();

    let app_state = AppState {
        pair_manager: Default::default(),
        openai_client,
        _db_pool: db_pool,
    };

    let state = Arc::new(app_state);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/get_people_count", get(get_people_count))
        .route("/api/report", post(report))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
