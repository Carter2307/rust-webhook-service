use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
}