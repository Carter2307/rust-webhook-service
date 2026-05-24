use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    repositories::{
        destination::DestinationRepository, event::EventRepository, webhook::WebhookRepository,
    },
    workers::job::DeliveryJob,
};

pub struct Dispatcher {
    redis: MultiplexedConnection,
    pool: PgPool,
}

impl Dispatcher {
    pub fn new(redis: MultiplexedConnection, pool: PgPool) -> Self {
        Self { redis, pool }
    }

    fn get_repo(&self) -> (WebhookRepository, DestinationRepository, EventRepository) {
        let wb_repo_pool = self.pool.clone();
        let dest_repo_pool = self.pool.clone();
        let event_repo_pool = self.pool.clone();

        (
            WebhookRepository { pool: wb_repo_pool },
            DestinationRepository {
                pool: dest_repo_pool,
            },
            EventRepository {
                pool: event_repo_pool,
            },
        )
    }

    pub async fn run(&self) -> Result<(), ServiceError> {
        let (wb_repo, dest_repo, event_repo) = self.get_repo();
        let mut redis = self.redis.clone();

        println!("Dispatcher started");
        loop {
            let (_, event_id): (String, String) = redis::cmd("BRPOP")
                .arg("events:queue")
                .arg(0) // 0 = bloque indéfiniment
                .query_async(&mut redis)
                .await?;

            // 1. Récupérer l'évènemnt
            println!("Dispatcher: Getting Event...");
            let event_result = event_repo
                .find_by_id(Uuid::parse_str(event_id.as_str())?)
                .await?;

            if let Some(event) = event_result {
                // 2. Récupérer les destinations à partir de l'id du webhook disponible dans l'évent
                println!("Dispatcher: Getting destination...");
                let results = dest_repo.find_by_webhook_id(event.webhook_id).await?;
                let client = reqwest::Client::new();

                // 3. Créer des Job pour chaque destination
                println!("Dispatcher: Create Job...");
                for dest in results {
                    let job = DeliveryJob {
                        event: event.clone(),
                        destination: dest,
                        http_client: client.clone(),
                        db: self.pool.clone(),
                    };
                    
                    let _ = tokio::spawn(async move {
                        job.execute().await
                    });
                }
            }
        }
    }
}
