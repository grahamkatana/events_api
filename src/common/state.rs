use crate::common::email::EmailSender;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub mailer: Arc<dyn EmailSender>,
}

pub type SharedState = AppState;