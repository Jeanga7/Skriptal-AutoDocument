use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Password hashing failed")
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    verify(password, hashed).expect("Password verification failed")
}
