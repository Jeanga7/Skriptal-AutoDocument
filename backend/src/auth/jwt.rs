use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation, TokenData};
use serde::{Serialize, Deserialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_jwt(user_id: &str) -> String {
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: 10000000000, // Date d'expiration du token
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("Token creation failed");

    token
}

pub fn decode_jwt(token: &str) -> TokenData<Claims> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
        .expect("Token decoding failed")
}
