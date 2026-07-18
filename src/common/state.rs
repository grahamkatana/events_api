use crate::common::email::EmailSender;
use crate::common::storage::Storage;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub mailer: Arc<dyn EmailSender>,
    pub storage: Storage,
    // A "broadcast" channel: any number of tasks can subscribe(), and
    // every value sent goes to ALL current subscribers — perfect for
    // "notify every connected WebSocket client at once."
    pub ws_tx: broadcast::Sender<String>,
}

pub type SharedState = AppState;