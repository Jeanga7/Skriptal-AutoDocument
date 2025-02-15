use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use sqlx::{types::Uuid as SqlxUuid, PgPool};

use crate::{
    auth::{jwt::generate_jwt, middleware::verify_jwt, password, User},
    utils::errors::error_response,
};

use super::{
    LoginRequest, LoginResponse, LogoutRequest, RegisterRequest, RegisterResponse,
    UpdateProfileRequest, UserProfile,
};

/// Creates and returns a router with routes for user.
pub fn routes(db: PgPool) -> Router<PgPool> {
    let user_routes = Router::new()
        .route("/profile", get(get_profile))
        .route("/update", post(update_profile))
        .route("/delete", post(delete_account))
        .layer(middleware::from_fn_with_state(db.clone(), verify_jwt));

    // Routeur principal
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/logout", post(logout))
        .nest("/user", user_routes)
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
        "INSERT INTO users (username, email, password_hash, first_name, last_name) 
         VALUES ($1, $2, $3, $4, $5)",
        payload.username,
        payload.email,
        password_hash,
        payload.first_name,
        payload.last_name
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
    // Chercher un utilisateur soit par son email soit par son nom d'utilisateur
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1 OR username = $1",
        payload.email
    )
    .fetch_one(&db)
    .await
    .map_err(|_| {
        error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid email or username or password",
        )
    })?;

    // Vérifier le mot de passe
    if !password::verify_password(&payload.password, &user.password_hash) {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid email or username or password",
        ));
    }

    // Générer le token JWT
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

pub async fn get_profile(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<SqlxUuid>,
) -> Result<Json<UserProfile>, impl IntoResponse> {
    let user = sqlx::query_as!(
        UserProfile,
        "SELECT email, username, first_name, last_name, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&pool)
    .await;

    match user {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err((StatusCode::NOT_FOUND, "User not found")),
    }
}

pub async fn update_profile(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<SqlxUuid>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Response {
    let password_hash = if let Some(password) = payload.password {
        Some(password::hash_password(&password))
    } else {
        None
    };

    let result = sqlx::query!(
        "UPDATE users SET 
            username = COALESCE($1, username),
            first_name = COALESCE($2, first_name),
            last_name = COALESCE($3, last_name),
            updated_at = NOW(),
            password_hash = COALESCE($4, password_hash)
        WHERE id = $5",
        payload.username,
        payload.first_name,
        payload.last_name,
        password_hash,
        user_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, "Profile updated successfully").into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update profile",
        ),
    }
}

pub async fn delete_account(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<SqlxUuid>,
) -> Response {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, "Account deleted successfully").into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete account",
        ),
    }
}
