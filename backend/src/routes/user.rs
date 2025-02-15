use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use sqlx::{types::Uuid as SqlxUuid, PgPool};

use crate::{
    auth::{jwt::generate_jwt, middleware::verify_jwt, password, User},
    utils::{errors::AppError, is_valid_email},
};

use super::{
    LoginRequest, LoginResponse, LogoutRequest, RegisterRequest, RegisterResponse,
    UpdateProfileRequest, UserProfile,
};

// ===========================================================================
//                                UserRepository
// ===========================================================================
/// Repository for managing user data in the database.
#[derive(Clone)]
struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    // Function to create a new user in the database
    async fn create(&self, payload: RegisterRequest) -> Result<(), AppError> {
        let password_hash = password::hash_password(&payload.password);

        sqlx::query!(
            "INSERT INTO users (username, email, password_hash, first_name, last_name) 
             VALUES ($1, $2, $3, $4, $5)",
            payload.username,
            payload.email,
            password_hash,
            payload.first_name,
            payload.last_name
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create user: {}", e),
            )
        })?;

        Ok(())
    }

    /// Asynchronously finds a user by their identity (email or username) and
    /// returns a Result containing the found User or an AppError if the credentials are invalid.
    async fn find_by_identity(&self, identity: &str) -> Result<User, AppError> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1 OR username = $1",
            identity
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AppError::unauthorized("Invalid credentials"))
    }

    /// Retrieves the profile of a user with the specified `user_id` from the database.
    async fn get_profile(&self, user_id: SqlxUuid) -> Result<UserProfile, AppError> {
        sqlx::query_as!(
            UserProfile,
            "SELECT email, username, first_name, last_name, created_at 
             FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AppError::not_found("User not found"))
    }

    /// Updates the user profile with the given `user_id` and `payload`.
    async fn update_profile(
        &self,
        user_id: SqlxUuid,
        payload: UpdateProfileRequest,
    ) -> Result<(), AppError> {
        let password_hash = match payload.password.as_deref() {
            Some(password) => Some(password::hash_password(password)),
            None => None,
        };

        sqlx::query!(
            "UPDATE users SET 
                username = COALESCE($1, username),
                first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                password_hash = COALESCE($4, password_hash),
                updated_at = NOW()
             WHERE id = $5",
            payload.username,
            payload.first_name,
            payload.last_name,
            password_hash,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }

    /// Deletes a user with the specified `user_id` from the `users` table.
    async fn delete(&self, user_id: SqlxUuid) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }
}


// ===========================================================================
//                                TokenRepository
// ===========================================================================

#[derive(Clone)]
struct TokenRepository {
    pool: PgPool,
}

/// Asynchronously revokes a token by inserting it into the `revoked_tokens` table.
///  If the token already exists in the table, it will not be inserted again.
impl TokenRepository {
    async fn revoke(&self, token: &str) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO revoked_tokens (token) VALUES ($1) 
             ON CONFLICT (token) DO NOTHING",
            token
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(())
    }
}

// ===========================================================================
//                                ROUTES
// ===========================================================================
/// Creates and returns the router for handling routes in the app.
pub fn routes(db: PgPool) -> Router<PgPool> {
    let user_repo = UserRepository { pool: db.clone() };
    let token_repo = TokenRepository { pool: db.clone() };

    // protected routes, need JWT token
    let user_routes = Router::new()
        .route("/profile", get(get_profile))
        .route("/update", post(update_profile))
        .route("/delete", post(delete_account))
        .layer(middleware::from_fn_with_state(db.clone(), verify_jwt));

    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/logout", post(logout))
        .nest("/user", user_routes)
        .with_state(db)
        .layer(Extension(user_repo))
        .layer(Extension(token_repo))
}

// ===========================================================================
//                                CRUD OPERATIONS
// ===========================================================================

/// Registers a user with the provided username, password, and email (optional first and last name ).
async fn register_user(
    Extension(repo): Extension<UserRepository>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(AppError::bad_request("Username/password cannot be empty"));
    }

    if !is_valid_email(&payload.email) {
        return Err(AppError::bad_request("Invalid email format"));
    }

    repo.create(payload).await?;
    Ok((StatusCode::CREATED, Json(RegisterResponse::success())))
}

/// Asynchronously logs in a user.
async fn login_user(
    Extension(repo): Extension<UserRepository>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = repo.find_by_identity(&payload.email).await?;

    if !password::verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    let token = generate_jwt(&user.id.to_string());
    Ok(Json(LoginResponse { token }))
}

/// Logout function that revokes the provided token from the TokenRepository.
async fn logout(
    Extension(repo): Extension<TokenRepository>,
    Json(payload): Json<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    repo.revoke(&payload.token).await?;
    Ok((StatusCode::OK, "Logged out successfully"))
}

/// Asynchronously retrieves the profile of a user.
async fn get_profile(
    Extension(repo): Extension<UserRepository>,
    Extension(user_id): Extension<SqlxUuid>,
) -> Result<Json<UserProfile>, AppError> {
    let profile = repo.get_profile(user_id).await?;
    Ok(Json(profile))
}

/// Updates the user's profile.
async fn update_profile(
    Extension(repo): Extension<UserRepository>,
    Extension(user_id): Extension<SqlxUuid>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    repo.update_profile(user_id, payload).await?;
    Ok((StatusCode::OK, "Profile updated"))
}

/// Deletes an account.
async fn delete_account(
    Extension(repo): Extension<UserRepository>,
    Extension(user_id): Extension<SqlxUuid>,
) -> Result<impl IntoResponse, AppError> {
    repo.delete(user_id).await?;
    Ok((StatusCode::OK, "Account deleted"))
}
