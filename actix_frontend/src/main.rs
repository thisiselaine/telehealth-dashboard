use actix_files as fs; // For static file serving
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde_json::json;

async fn api_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "message": "Hello from API" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api", web::get().to(api_handler)) // Example API endpoint
            .service(fs::Files::new("/", "./static").index_file("index.html")) // Serve static files
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
