use actix_web::{post, HttpResponse, web};
use crate::services::auth_service::{register_user, login_user};
use crate::db::models::User;

#[post("/register")]
async fn register(data: web::Json<User>) -> HttpResponse {
    match register_user(data.into_inner()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/login")]
async fn login(data: web::Json<User>) -> HttpResponse {
    match login_user(data.into_inner()).await {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}
