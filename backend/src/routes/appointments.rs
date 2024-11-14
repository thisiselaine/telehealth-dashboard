use actix_web::{post, get, HttpResponse, web};
use crate::services::appointment_service::{create_appointment, list_appointments};
use crate::db::models::Appointment;
use uuid::Uuid;

#[post("/appointments")]
async fn book_appointment(data: web::Json<Appointment>) -> HttpResponse {
    match create_appointment(data.into_inner()).await {
        Ok(appointment) => HttpResponse::Ok().json(appointment),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/appointments/{provider_id}")]
async fn provider_appointments(provider_id: web::Path<Uuid>) -> HttpResponse {
    match list_appointments(provider_id.into_inner()).await {
        Ok(appointments) => HttpResponse::Ok().json(appointments),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
