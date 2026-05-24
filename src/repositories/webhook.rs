use sqlx::PgPool;
use uuid::Uuid;

use crate::models::webhook::Webhook;

pub struct WebhookRepository {
    pub pool: PgPool,
}

impl WebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(self: &Self, wb: Webhook) -> Result<Webhook, sqlx::Error> {
        let webhook = sqlx::query_as!(
            Webhook,
            "INSERT INTO webhooks (id, secret, name, description, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *",
            wb.id,
            wb.secret,
            wb.name,
            wb.description,
            wb.created_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(webhook)
    }

    pub async fn find_by_id(self: &Self, id: Uuid) -> Result<Option<Webhook>, sqlx::Error> {
        let result = sqlx::query_as!(Webhook, "SELECT * FROM webhooks WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn update(self: &Self, wb: Webhook) -> Result<Option<Webhook>, sqlx::Error> {
        let result = sqlx::query_as!(
            Webhook,
            "UPDATE webhooks
            SET name = $2 , description = $3
            WHERE id = $1
            RETURNING *",
            wb.id,
            wb.name,
            wb.description
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM webhooks WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
