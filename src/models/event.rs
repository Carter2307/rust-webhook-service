use std::clone;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "events_state", rename_all = "PascalCase")]
pub enum EventState {
    Pending,
    Processed,
    Partial,
    Error,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, Clone)]
pub struct Event  {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub data: Value,  // Peut contenir n'importe quoi
    pub state: EventState,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}