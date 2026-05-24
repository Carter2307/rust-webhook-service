use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub secret: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}