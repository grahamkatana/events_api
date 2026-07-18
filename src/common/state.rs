use crate::common::email::EmailSender;
use crate::common::storage::Storage;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub mailer: Arc<dyn EmailSender>,
    pub storage: Storage,
}

pub type SharedState = AppState;