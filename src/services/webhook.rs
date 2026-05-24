use std::error::Error;

use chrono::{DateTime, Utc};
use redis::{AsyncCommands, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, database};
use uuid::Uuid;

use crate::{
    error::ServiceError, models::{
        destination::{Destination, DestinationState},
        event::{Event, EventState},
        webhook::Webhook,
    }, repositories::{
        destination::{self, DestinationRepository},
        event::EventRepository,
        webhook::WebhookRepository,
    }
};

#[derive(Serialize, Deserialize)]
pub struct CreateWebhookInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub destinations: Vec<CreateDestinationInput>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateWebhookInput {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteWebhookInput {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct CreateDestinationInput {
    pub url: String,
    pub api_key: Option<String>,
}

pub struct WebhookService {
    pub pool: PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub wb_repo: WebhookRepository,
    pub dest_reop: DestinationRepository,
    pub event_repo: EventRepository,
}

impl WebhookService {
    pub fn new(conn: PgPool, redis_conn: MultiplexedConnection) -> Self {
        let wb_repo_pool = conn.clone();
        let dest_repo_pool = conn.clone();
        let event_repo_pool = conn.clone();

        Self {
            pool: conn,
            redis:  redis_conn,
            wb_repo: WebhookRepository { pool: wb_repo_pool },
            dest_reop: DestinationRepository {
                pool: dest_repo_pool,
            },
            event_repo: EventRepository {
                pool: event_repo_pool,
            },
        }
    }

    pub async fn create(&self, wb_input: CreateWebhookInput) -> Result<Webhook, sqlx::Error> {
        let id = Uuid::new_v4();
        let secret = Uuid::new_v4().to_string();

        // Transaction: Si l'une des opération qui suivent en base echoue, alors rien n'est stocké en base et retourne une erreur
        let mut tx = self.pool.begin().await?;

        // 3. Créer le webhook directement avec la transaction
        let wb = sqlx::query_as!(
            Webhook,
            r#"INSERT INTO webhooks (id, secret, name, description, created_at)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
            id,
            secret,
            wb_input.name,
            wb_input.description,
            Utc::now()
        )
        .fetch_one(&mut *tx)
        .await?;

        // 4. Créer les destinations
        for d in wb_input.destinations {
            sqlx::query!(
                "INSERT INTO destinations (id, webhook_id, url, api_key, state, retry_count, created_at)
                 VALUES ($1, $2, $3, $4, 'Pending', 0, $5)",
                Uuid::new_v4(),
                id,
                d.url,
                d.api_key,
                Utc::now()
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(wb)
    }

    pub async fn update(
        &self,
        id: Uuid,
        wb_input: UpdateWebhookInput,
    ) -> Result<Option<Webhook>, sqlx::Error> {
        let result = self.wb_repo.find_by_id(id).await?;

        if let Some(wb) = result {
            let w = self
                .wb_repo
                .update(Webhook {
                    id: wb.id,
                    secret: wb.secret,
                    name: wb_input.name,
                    description: wb_input.description,
                    created_at: wb.created_at,
                })
                .await?;

            Ok(w)
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        let result = self.wb_repo.find_by_id(id).await?;

        if let Some(_) = result {
            self.wb_repo.delete(id).await?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub async fn trigger(&self, webhook_id: Uuid, data: Value) -> Result<Event, ServiceError> {
        //1. Créer l'Event en base (state: Pending)
        let event_id = Uuid::new_v4();
        let result = self
            .event_repo
            .create(Event {
                id: event_id,
                webhook_id,
                data,
                state: EventState::Pending,
                created_at: Utc::now(),
                updated_at: None
            })
            .await?;

        //2. Pousser l'event_id dans Redis
        let mut redis = self.redis.clone();
        let _: () = redis.lpush("events:queue", result.id.to_string()).await?;
        
        //3. Retourner l'Event créé
        Ok(result)
    }
}
