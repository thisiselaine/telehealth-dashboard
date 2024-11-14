use actix_web::{get, post, HttpResponse, web};
use crate::services::provider_service::{get_providers, add_provider};
use crate::db::models::Provider;

#[post("/providers")]
async fn create_provider(data: web::Json<Provider>) -> HttpResponse {
    match add_provider(data.into_inner()).await {
        Ok(provider) => HttpResponse::Ok().json(provider),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/providers")]
async fn list_providers(specialty: web::Query<String>) -> HttpResponse {
    match get_providers(specialty.into_inner()).await {
        Ok(providers) => HttpResponse::Ok().json(providers),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
