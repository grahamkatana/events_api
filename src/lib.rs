pub mod auth;
pub mod common;
pub mod events;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use common::email::SmtpMailer;
use common::openapi::ApiDoc;
use common::state::{AppState, SharedState};
use common::storage::Storage;
use common::video::VideoClient;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Builds the fully wired application: connects to Postgres, sets up
/// the mailer/storage/broadcast channel, and assembles every route.
/// `main.rs` calls this to run the real server; our integration tests
/// call it too, so tests exercise the EXACT same app, not a stand-in.
pub async fn build_app(database_url: &str, jwt_secret: String) -> Router {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres");

    let mailer = Arc::new(SmtpMailer::from_env());
    let storage = Storage::from_env().await;
    let video = VideoClient::from_env();
    let (ws_tx, _) = tokio::sync::broadcast::channel::<String>(100);

    let shared: SharedState = AppState {
        db: pool,
        jwt_secret,
        mailer,
        storage,
        ws_tx,
        video,
    };

    events::routes::build_router(shared.clone())
        .merge(auth::routes::build_router(shared.clone()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        //Handle cors
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}