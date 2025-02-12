use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn generate_jwt(user_id: &str) -> String {
    let expiration = chrono::Utc::now().timestamp() as usize + 3600; // 1h

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("Failed to create JWT")
}
