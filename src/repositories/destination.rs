use crate::models::{destination::Destination, destination::DestinationState};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct DestinationRepository {
    pub pool: PgPool,
}

impl DestinationRepository {
    pub fn new(pool: PgPool) -> Self {
        DestinationRepository { pool }
    }

    pub async fn create(&self, d: Destination) -> Result<Destination, sqlx::Error> {
        let destination = sqlx::query_as!(
            Destination,
            r#"INSERT INTO destinations (id, webhook_id, url, api_key, state, retry_count, next_retry_at, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5::destinations_state, $6, NULL, $7, NULL)
               RETURNING
                 id,
                 webhook_id,
                 url,
                 api_key,
                 state as "state: DestinationState",
                 retry_count,
                 next_retry_at,
                 created_at,
                 updated_at"#,
            d.id,
            d.webhook_id,
            d.url,
            d.api_key,
            d.state as DestinationState,
            d.retry_count as _,
            d.created_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(destination)
    }

    pub async fn find_by_webhook_id(&self, webhook_id: Uuid) -> Result<Vec<Destination>, sqlx::Error> {
        let result = sqlx::query_as!(
            Destination,
            r#"SELECT
                 id,
                 webhook_id,
                 url,
                 api_key,
                 state as "state: DestinationState",
                 retry_count,
                 next_retry_at,
                 created_at,
                 updated_at
               FROM destinations
               WHERE webhook_id = $1"#,
               webhook_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update(&self, d: Destination) -> Result<Option<Destination>, sqlx::Error> {
        let result = sqlx::query_as!(
            Destination,
            r#"UPDATE destinations
               SET url = $2, api_key = $3, state = $4::destinations_state
               WHERE id = $1
               RETURNING
                 id,
                 webhook_id,
                 url,
                 api_key,
                 state as "state: DestinationState",
                 retry_count,
                 next_retry_at,
                 created_at,
                 updated_at"#,
            d.id,
            d.url,
            d.api_key,
            d.state as DestinationState
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM destinations WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
