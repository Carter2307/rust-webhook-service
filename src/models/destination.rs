use std::clone;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "destinations_state", rename_all = "PascalCase")]
pub enum DestinationState {
    Pending,
    Reached,
    Error,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, Clone)]
pub struct Destination {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub url: String,
    pub api_key: Option<String>,
    pub state: DestinationState,
    pub retry_count: i64,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
