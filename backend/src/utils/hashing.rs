use argon2::{self, Config};

pub fn hash_password(password: &str) -> String {
    let salt = b"randomsalt";
    argon2::hash_encoded(password.as_bytes(), salt, &Config::default()).unwrap()
}

pub fn verify_password(hashed: &str, password: &str) -> bool {
    argon2::verify_encoded(hashed, password.as_bytes()).unwrap_or(false)
}
