use std::sync::Arc;
use sqlx::{postgres::PgPoolOptions};
use crate::{http::routes::create_router, services::webhook::WebhookService, workers::dispatcher::Dispatcher};

mod config;
mod http;
mod models;
mod queue;
mod repositories;
mod services;
mod workers;
mod error;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // 1. PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Impossible de se connecter à PostgreSQL");

    // 2. Migrations
    sqlx::migrate!("./src/db/migrations")
        .run(&pool)
        .await
        .expect("Erreur lors des migrations");

    // 3. Redis
    let redis_client = redis::Client::open(
        std::env::var("REDIS_URL").unwrap()
    ).unwrap();

    // Connexion multiplexée pour le service HTTP (LPUSH, non-bloquant)
    let service_redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    // Connexion dédiée pour le worker (BRPOP bloquant — ne doit PAS être partagée)
    let worker_redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    // 4. Service
    let service = Arc::new(
        WebhookService::new(pool.clone(), service_redis_conn)
    );

    // 5. Router
    let server_service = service.clone();
    let server = tokio::spawn(async move {
        let app = create_router(server_service);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:4400")
            .await
            .unwrap();
        println!("Listening on http://localhost:4400");
        axum::serve(listener, app).await.unwrap();
    });

    // 6. Dispatcher
    let worker = tokio::spawn(async move {
        let dispatcher = Dispatcher::new(worker_redis_conn, pool);
        dispatcher.run().await.unwrap();
    });

    tokio::join!(server, worker);
}