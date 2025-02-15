pub mod errors;

pub fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
    re.is_match(email)
}