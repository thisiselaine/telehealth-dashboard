use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::services::auth_service::Claims;

pub async fn auth_middleware(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret("secret_key".as_ref()),
        &Validation::new(Algorithm::HS256)
    );

    match decoded {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => Err(actix_web::error::ErrorUnauthorized("Unauthorized")),
    }
}
