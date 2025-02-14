use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
// use thiserror::Error;

/* #[derive(Error, Debug)]
pub enum LoginError {
    #[error("User not found")]
    UserNotFound(#[from] sqlx::Error),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
 */

pub fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}
