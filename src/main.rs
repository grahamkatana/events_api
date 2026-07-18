mod auth;
mod common;
mod events;

use axum::extract::DefaultBodyLimit;
use common::email::SmtpMailer;
use common::state::{AppState, SharedState};
use common::storage::Storage;
use std::sync::Arc;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Every log line goes to BOTH the console and a file, so you don't
    // have to choose one or the other while developing.
    std::fs::create_dir_all("storage/logs").expect("failed to create storage/logs directory");

    // `rolling::daily` starts a fresh file each day, named with the
    // date — e.g. storage/logs/events_api.log.2026-07-18 — so logs
    // don't grow into one unbounded file forever.
    let file_appender = tracing_appender::rolling::daily("storage/logs", "events_api.log");

    // Writing to a file is comparatively slow; `non_blocking` hands
    // log lines off to a background thread so a handler is never
    // stuck waiting on disk I/O just to log something. `_guard` must
    // stay alive for the program's whole lifetime — dropping it early
    // would silently stop flushing logs to disk.
    let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer()) // console, with colors
        .with(fmt::layer().with_writer(non_blocking_file).with_ansi(false)) // file, no color codes
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in .env");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    tracing::info!("Connected to database successfully");

    let mailer = Arc::new(SmtpMailer::from_env());
    let storage = Storage::from_env().await;

    let shared: SharedState = AppState {
        db: pool,
        jwt_secret,
        mailer,
        storage,
    };

    let app = events::routes::build_router(shared.clone())
        .merge(auth::routes::build_router(shared.clone()))
        // Caps every request body at 5MB — generous for a cover image,
        // but stops someone from uploading a 2GB file and tying up
        // memory/bandwidth. Applies to the whole app, not just uploads.
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
        // Logs every request automatically: method, path, status code,
        // and how long it took — no per-handler code needed for this.
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}