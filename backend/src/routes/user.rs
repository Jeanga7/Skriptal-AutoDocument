use axum::{Json, extract::State, routing::post, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::auth::jwt::generate_jwt;
use crate::auth::password::{hash_password, verify_password};
use crate::auth::User;

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    message: String,
}

async fn register_user(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    let password_hash = hash_password(&payload.password);

    sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)",
        payload.username,
        payload.email,
        password_hash
    )
    .execute(&pool)
    .await
    .expect("Failed to insert user");

    Json(RegisterResponse {
        message: "User created successfully".to_string(),
    })
}

pub fn routes(pool: PgPool) -> Router {
    Router::new().route("/register", post(register_user)).with_state(pool)
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login_user(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Json<LoginResponse> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        payload.email
    )
    .fetch_one(&pool)
    .await
    .expect("User not found");

    if !verify_password(&payload.password, &user.password_hash) {
        panic!("Invalid credentials");
    }

    let token = generate_jwt(&user.id.to_string());
    Json(LoginResponse { token })
}
