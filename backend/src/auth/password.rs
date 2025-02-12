use bcrypt::{hash, verify, DEFAULT_COST};
/// Hashes the given password using the bcrypt algorithm with the default cost.
/// This function will panic if the password hashing fails.
pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Failed to hash password")
}

/// Verifies the given password against the given hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).expect("Failed to verify password")
}

