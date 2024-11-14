pub fn validate_email(email: &str) -> bool {
    regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap().is_match(email)
}

pub fn validate_password(password: &str) -> bool {
    password.len() >= 8
}
