use sqlx::{types::Uuid, FromRow};

pub mod jwt;
pub mod password;
pub mod middleware;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

