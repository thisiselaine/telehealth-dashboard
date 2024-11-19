mod find_providers;
use find_providers::{geocode_address, find_health_providers};

use actix_files as fs; // For static file serving
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
// use serde_json::json;
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParams {
    zip: String,
}

// Handler for the `/services` endpoint
async fn services_handler(query: web::Query<QueryParams>) -> impl Responder {
    let zip = &query.zip;

    // Load the Google Maps API key from environment
    let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
        .expect("GOOGLE_MAPS_API_KEY must be set in environment");

    // Attempt to geocode the zip code
    let coordinates = match geocode_address(zip, &api_key).await {
        Ok(coords) => coords,
        Err(err) => {
            eprintln!("Geocoding failed: {}", err);
            return HttpResponse::InternalServerError().body("Failed to geocode zip code");
        }
    };

    // Find health providers near the coordinates
    let providers = match find_health_providers(&coordinates, 10000, &api_key).await {
        Ok(providers) => providers,
        Err(err) => {
            eprintln!("Failed to find health providers: {}", err);
            return HttpResponse::InternalServerError().body("Failed to fetch health services");
        }
    };

    // Return the list of health providers as JSON
    HttpResponse::Ok().json(providers)
}

// async fn api_handler() -> impl Responder {
//     HttpResponse::Ok().json(serde_json::json!({ "message": "Hello from API" }))
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .route("/services", web::get().to(services_handler)) // Endpoint for health services
            .service(fs::Files::new("/", "./static").index_file("index.html")) // Serve static files
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
