use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
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


pub fn error_response(status: StatusCode, message: &str) -> impl IntoResponse {
    (status, Json(json!({ "error": message })))
}
