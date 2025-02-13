use crate::auth::{jwt::generate_jwt, middleware::verify_jwt, password, User};
use crate::utils::errors::error_response;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{extract::State, routing::post, Json, Router};
use sqlx::PgPool;

use super::{LoginRequest, LoginResponse, LogoutRequest, RegisterRequest, RegisterResponse};
/// Creates and returns a router with routes for user registration and login.
pub fn routes(db: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/logout", post(logout))
        .route(
            "/protected",
            get(protected_route).layer(middleware::from_fn_with_state(db.clone(), verify_jwt)),
        )
        .with_state(db)
}

/// Registers a new user.
async fn register_user(
    State(pool): State<sqlx::PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Username or password cannot be empty",
        ));
    }

    let password_hash = password::hash_password(&payload.password);

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)",
        payload.username,
        payload.email,
        password_hash
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok((
            StatusCode::CREATED,
            Json(RegisterResponse {
                message: "User created successfully".to_string(),
            }),
        )),
        Err(e) => {
            eprintln!("Failed to execute query: {:?}", e);
            Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create user",
            ))
        }
    }
}

/// Asynchronously logs in a user.
/// The login response JSON containing the generated token.
pub async fn login_user(
    State(db): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, impl IntoResponse> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", payload.email)
        .fetch_one(&db)
        .await
        .map_err(|_| error_response(StatusCode::UNAUTHORIZED, "Invalid email or password"))?;

    if !password::verify_password(&payload.password, &user.password_hash) {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid email or password",
        ));
    }

    let token = generate_jwt(&user.id.to_string());
    Ok(Json(LoginResponse { token }))
}
/// Logs out a user by revoking the provided token and storing it in the `revoked_tokens` table in the database.
pub async fn logout(
    State(pool): State<PgPool>,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO revoked_tokens (token) VALUES ($1) ON CONFLICT (token) DO NOTHING",
        payload.token
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, "User logged out successfully"),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error logging out"),
    }
}

async fn protected_route() -> impl IntoResponse {
    "You have access to this protected route!".into_response()
}
