use sqlx::{types::Uuid, FromRow};

pub mod jwt;
pub mod password;
pub mod middleware;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>, 
    pub last_name: Option<String>,  
    pub password_hash: String,
    pub role: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,  
}

