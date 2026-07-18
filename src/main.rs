mod auth;
mod common;
mod events;

use common::email::SmtpMailer;
use common::state::{AppState, SharedState};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in .env");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    println!("Connected to database successfully");

    let mailer = Arc::new(SmtpMailer::from_env());

    let shared: SharedState = AppState {
        db: pool,
        jwt_secret,
        mailer,
    };

    let app = events::routes::build_router(shared.clone())
        .merge(auth::routes::build_router(shared.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}