use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use serde_json::Value;
use uuid::Uuid;

use crate::services::webhook::{
    CreateWebhookInput, DeleteWebhookInput, UpdateWebhookInput, WebhookService,
};

pub async fn create_webhook(
    State(service): State<Arc<WebhookService>>,
    Json(input): Json<CreateWebhookInput>,
) -> impl IntoResponse {
    match service.create(input).await {
        Ok(webhook) => (StatusCode::CREATED, Json(webhook)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_webhook(
    State(service): State<Arc<WebhookService>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateWebhookInput>,
) -> impl IntoResponse {
    match service.update(id, input).await {
        Ok(webhook) => (StatusCode::OK, Json(webhook)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_webhook(
    State(service): State<Arc<WebhookService>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.delete(id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn trigger_webhook(
    State(service): State<Arc<WebhookService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match service.trigger(id, payload).await {
        Ok(webhook) => (StatusCode::OK, Json(webhook)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub fn create_router(service: Arc<WebhookService>) -> Router {
    Router::new()
        .route("/webhooks", post(create_webhook))
        .route("/webhooks/:id", patch(update_webhook))
        .route("/webhooks/:id", delete(delete_webhook))
        .route("/webhooks/:id", post(trigger_webhook))
        // .route("/webhooks/:webhookId/events", get(method_router))
        // .route("/webhooks/:webhookId/events/:eventId/replay", post(method_router))
        // .route("/webhooks/:webhookId/metrics", get(method_router))
        .with_state(service)
}
