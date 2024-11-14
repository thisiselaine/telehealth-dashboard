use crate::db::models::User;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use argon2::{self, Config};
use crate::db::queries::{create_user, get_user_by_email};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn register_user(user: User) -> Result<User, String> {
    let hashed_password = hash_password(&user.hashed_password);
    let mut new_user = user.clone();
    new_user.hashed_password = hashed_password;

    create_user(&pool, new_user).await.map_err(|_| "Error registering user".into())
}

pub async fn login_user(email: String, password: String) -> Result<String, String> {
    if let Some(user) = get_user_by_email(&pool, &email).await.ok() {
        let is_password_valid = verify_password(&user.hashed_password, &password);
        if is_password_valid {
            let claims = Claims {
                sub: user.id.to_string(),
                exp: 10000000000,
            };
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret_key".as_ref())).unwrap();
            Ok(token)
        } else {
            Err("Invalid credentials".into())
        }
    } else {
        Err("User not found".into())
    }
}
