use crate::models::event::{Event, EventState};
use sqlx::PgPool;
use uuid::Uuid;

pub struct EventRepository {
    pub pool: PgPool,
}

impl EventRepository {
    pub fn new(pool: PgPool) -> Self {
        EventRepository { pool }
    }

    pub async fn create(&self, e: Event) -> Result<Event, sqlx::Error> {
        let event = sqlx::query_as!(
            Event,
            r#"INSERT INTO events (id, webhook_id, data, state, created_at, updated_at)
               VALUES ($1, $2, $3, $4::events_state, $5, NULL)
               RETURNING
                 id,
                 webhook_id,
                 data,
                 state as "state: EventState",
                 created_at,
                 updated_at"#,
            e.id,
            e.webhook_id,
            e.data,
            e.state as EventState,
            e.created_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(event)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Event>, sqlx::Error> {
        let result = sqlx::query_as!(
            Event,
            r#"SELECT
                 id,
                 webhook_id,
                 data,
                 state as "state: EventState",
                 created_at,
                 updated_at
               FROM events
               WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_state(&self, e: Event) -> Result<Option<Event>, sqlx::Error> {
        let result = sqlx::query_as!(
            Event,
            r#"UPDATE events
               SET state = $2::events_state
               WHERE id = $1
               RETURNING
                 id,
                 webhook_id,
                 data,
                 state as "state: EventState",
                 created_at,
                 updated_at"#,
            e.id,
            e.state as EventState
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }
}
