use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};

pub mod generate;
pub mod user;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub token: String,
}

#[derive(Debug, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

