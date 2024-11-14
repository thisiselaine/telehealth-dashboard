use actix_web::{post, web, HttpResponse, Responder};
use crate::services::notifications_service::send_notification;
use crate::db::models::Notification;

#[post("/notifications/send")]
async fn send_notification_route(notification: web::Json<Notification>) -> impl Responder {
    match send_notification(&notification).await {
        Ok(_) => HttpResponse::Ok().json("Notification sent successfully"),
        Err(err) => HttpResponse::InternalServerError().json(format!("Failed to send notification: {}", err)),
    }
}

pub fn configure_notifications(cfg: &mut web::ServiceConfig) {
    cfg.service(send_notification_route);
}
