use std::time::Duration;

use chrono::Utc;
use sqlx::PgPool;

use crate::{
    error::ServiceError,
    models::{
        destination::{Destination, DestinationState},
        event::{Event, EventState},
    },
    repositories::{
        destination::DestinationRepository, event::EventRepository, webhook::WebhookRepository,
    },
};

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000; // 1 seconde

pub struct DeliveryJob {
    pub event: Event,
    pub destination: Destination,
    pub http_client: reqwest::Client,
    pub db: PgPool,
}

impl DeliveryJob {
    pub fn new(
        event: Event,
        destination: Destination,
        http_client: reqwest::Client,
        db: PgPool,
    ) -> Self {
        Self {
            event,
            destination,
            http_client,
            db,
        }
    }

    fn get_repo(&self) -> (WebhookRepository, DestinationRepository, EventRepository) {
        let wb_repo_pool = self.db.clone();
        let dest_repo_pool = self.db.clone();
        let event_repo_pool = self.db.clone();

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

    async fn update_event_state(&self) -> Result<(), ServiceError> {
        let (_, dest_repo, event_repo) = self.get_repo();

        // 1. Récupérer toute les destinations
        let destinations = dest_repo.find_by_webhook_id(self.event.webhook_id).await?;

        let reached_count = destinations
            .iter()
            .filter(|d| matches!(d.state, DestinationState::Reached))
            .count();

        let error_count = destinations
            .iter()
            .filter(|d| matches!(d.state, DestinationState::Error))
            .count();

        // 2. Appliquer les règles de mise à jours de l'event
        if reached_count == destinations.len() {
            // event -> Processed
            event_repo
                .update_state(Event {
                    state: EventState::Processed,
                    updated_at: Some(Utc::now()),
                    ..self.event.clone()
                })
                .await?;
        } else if error_count == destinations.len() {
            // event -> Error
            event_repo
                .update_state(Event {
                    state: EventState::Error,
                    updated_at: Some(Utc::now()),
                    ..self.event.clone()
                })
                .await?;
        } else {
            // event -> Partial
            event_repo
                .update_state(Event {
                    state: EventState::Partial,
                    updated_at: Some(Utc::now()),
                    ..self.event.clone()
                })
                .await?;
        }

        Ok(())
    }

    pub async fn execute(&self) -> Result<(), ServiceError> {
        let (_, dest_repo, _) = self.get_repo();

        println!("Job: Started...");
        for attempt in 0..MAX_RETRIES {
            // POST le payload vers destination.endpoint
            println!("Job: Try to reach destination...");
            let response = self
                .http_client
                .post(&self.destination.url)
                .json(&self.event.data)
                .send()
                .await;

            
            match response {
                Ok(r) if r.status().is_success() => {
                    // Si succès → mettre à jour destination.state à Reached
                     println!("Job: Destination reached successfully... {:?}", r);
                    dest_repo
                        .update(Destination {
                            state: DestinationState::Reached,
                            updated_at: Some(Utc::now()),
                            ..self.destination.clone()
                        })
                        .await?;

                    self.update_event_state().await?;
                    return Ok(());
                }
                Err(e) => println!("Error: {}", e),
                _ => {
                    // Si échec → retry avec backoff exponentiel
                    let factor: u64 = 2;
                    println!("{:?}", &response);


                    if attempt < MAX_RETRIES - 1 {
                        let delay = BASE_DELAY_MS * factor.pow(attempt);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        // mettre à jour next_retry_at
                    } else {
                        // Après max retries → mettre à jour destination.state à Error
                        println!("Job: Failled to reach destination...");
                        dest_repo
                            .update(Destination {
                                state: DestinationState::Error,
                                updated_at: Some(Utc::now()),
                                ..self.destination.clone()
                            })
                            .await?;

                        self.update_event_state().await?;
                    }
                }
            }
        }

        Ok(())
    }
}
