use actix_web::web;

// Define the modules
pub mod auth;
pub mod providers;
pub mod appointments;
pub mod education;
pub mod notifications;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    // Authentication routes
    cfg.service(auth::register);
    cfg.service(auth::login);
    cfg.service(auth::logout); // If logout is implemented

    // Providers routes
    cfg.service(providers::get_providers);
    cfg.service(providers::add_provider); // Optional: add a provider if needed

    // Appointments routes
    cfg.service(appointments::book_appointment);
    cfg.service(appointments::check_availability); // Optional: check availability

    // Educational content routes
    cfg.service(education::get_content);

    // Notifications routes
    cfg.service(notifications::get_notifications); // Fetch notifications
    cfg.service(notifications::send_notification); // Optional: send notifications
}
