mod find_providers;
use find_providers::{geocode_address, find_health_providers, Coordinates};

use actix_files as fs; // For static file serving
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde_json::json;
use serde::{Deserialize};


#[derive(Deserialize)]
struct QueryParams {
    zip: Option<String>,
    lat: Option<f64>,    // Optional latitude
    lng: Option<f64>,    // Optional longitude
}

// Handler for the `/services` endpoint
async fn services_handler(query: web::Query<QueryParams>) -> impl Responder {
    let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
        .expect("GOOGLE_MAPS_API_KEY must be set in environment");

    let coordinates = if let (Some(lat), Some(lng)) = (query.lat, query.lng) {
        // Use lat/lng if provided
        // println!("Using provided latitude and longitude: {}, {}", lat, lng);
        Coordinates { lat, lng }
    } else if let Some(zip) = query.zip.as_deref() {
        // Geocode the ZIP code if lat/lng not provided
        match geocode_address(zip, &api_key).await {
            Ok(coords) => coords,
            Err(err) => {
                eprintln!("Geocoding failed: {}", err);
                return HttpResponse::InternalServerError().body("Failed to geocode zip code");
            }
        }
    } else {
        // Return an error if neither are provided
        return HttpResponse::BadRequest().body("Please provide either a ZIP code or lat/lng");
    };

    let providers = match find_health_providers(&coordinates, 10000, &api_key).await {
        Ok(providers) => providers,
        Err(err) => {
            eprintln!("Failed to find health providers: {}", err);
            return HttpResponse::InternalServerError().body("Failed to fetch health services");
        }
    };

    HttpResponse::Ok().json(json!({
        "coordinates": coordinates,
        "providers": providers,
    }))
}

// Handler for the `/api-key` endpoint
async fn api_key_handler() -> impl Responder {
    let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
        .expect("GOOGLE_MAPS_API_KEY must be set in environment");
    HttpResponse::Ok().body(api_key)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .route("/api-key", web::get().to(api_key_handler)) // Endpoint to serve the API key
            .route("/services", web::get().to(services_handler)) // Endpoint for health services
            .service(fs::Files::new("/", "./static").index_file("index.html")) // Serve static files
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
