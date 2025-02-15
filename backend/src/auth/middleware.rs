use crate::auth::jwt::Claims;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::types::Uuid as SqlxUuid;
use sqlx::PgPool;
use std::env;

/// Alias pour les erreurs HTTP
type HttpResult = Result<Response, Response>;

pub struct UnauthorizedMessage(&'static str);

impl IntoResponse for UnauthorizedMessage {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.0).into_response()
    }
}

/// Middleware pour vérifier l'authentification avec JWT
pub async fn verify_jwt<B>(
    State(pool): State<sqlx::PgPool>,
    mut req: Request<B>,
    next: Next<B>,
) -> HttpResult {
    let token = extract_token(&req)?;

    // Vérifier si le token est révoqué
    if is_token_revoked(&pool, &token).await? {
        return Err((StatusCode::UNAUTHORIZED, "Token has been revoked").into_response());
    }

    let sub = validate_jwt(&token)?;
    let user_id = SqlxUuid::parse_str(&sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID in token").into_response())?;


    // Attacher l'ID utilisateur au contexte de la requête
    req.extensions_mut().insert(user_id);

    Ok(next.run(req).await)
}

/// Extrait le token JWT depuis l'en-tête Authorization
fn extract_token<B>(req: &Request<B>) -> Result<String, Response> {
    req.headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer ").map(String::from))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            )
                .into_response()
        })
}

/// Vérifie et décode le token JWT
fn validate_jwt(token: &str) -> Result<String, Response> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = Validation::default();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map(|token_data| token_data.claims.sub) // Récupère l'ID utilisateur
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token").into_response())
}

/// Vérifie si le token est révoqué dans la base de données
async fn is_token_revoked(pool: &PgPool, token: &str) -> Result<bool, Response> {
    let result = sqlx::query!(
        "SELECT 1 AS token_exists FROM revoked_tokens WHERE token = $1",
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?;

    Ok(result.is_some())
}
