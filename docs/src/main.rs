mod find_providers;
use find_providers::{geocode_address, find_health_providers};

use actix_files as fs; 
use actix_web::{get, post, web, App, HttpServer, HttpRequest, Responder, HttpResponse, Result};
use actix_web::cookie::{Cookie, CookieBuilder};
use actix_files::NamedFile;
use serde_json::json;
use serde::Deserialize;
use sqlx::SqlitePool;
use handlebars::Handlebars;

#[derive(Deserialize)]
struct QueryParams {
    zip: String,
}

#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
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


// Handler for the `/login` endpoint
#[post("/login")]
async fn login_handler(
    form: web::Form<LoginData>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let LoginData { username, password } = form.into_inner();

    // Query the database for the user
    let user_result = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = ?",
        username
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user_result {
        Ok(Some(user)) => {
            // Check if the password matches (no hashing)
            let is_valid = user.password_hash == password;

            if is_valid {
                // Set a cookie with the username on successful login
                let cookie = CookieBuilder::new("username", username.clone())
                    .path("/")
                    .finish();
                
                return HttpResponse::Ok()
                    .cookie(cookie)
                    .body("Login successful");
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}

// Serves the login page at /login
#[get("/login")]
async fn login(_req: HttpRequest) -> Result<NamedFile> {
    NamedFile::open_async("./static/login.html").await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("File open error: {}", e))
    })
}

// Handler for the `/register` endpoint
#[post("/register")]
async fn register_handler(
    form: web::Form<LoginData>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let LoginData { username, password } = form.into_inner();

    // Ensure username and password are provided
    if username.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest().body("Username and password are required");
    }

    // Check if the username already exists
    let existing_user = sqlx::query!("SELECT id FROM users WHERE username = ?", username)
        .fetch_optional(pool.get_ref())
        .await;

    // Handle the result
    match existing_user {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().body("Username already exists");
        }
        Ok(None) => {
            // Insert the new user into the database
            let result = sqlx::query!(
                "INSERT INTO users (username, password_hash) VALUES (?, ?)",
                username,
                password
            )
            .execute(pool.get_ref())
            .await;

            match result {
                Ok(res) if res.rows_affected() == 1 => HttpResponse::Created().body("User created"),
                _ => HttpResponse::InternalServerError().body("Failed to create user"),
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}

// Serves the register page at /register
#[get("/register")]
async fn register(_req: HttpRequest) -> Result<NamedFile> {
    NamedFile::open_async("./static/register.html").await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("File open error: {}", e))
    })
}

// Handler for the `/logout` endpoint
#[post("/logout")]
async fn logout(_req: HttpRequest) -> impl Responder {
    // Clear the username cookie
    let cookie = Cookie::build("username", "")
        .path("/")
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .body("Logged out")
}

// Serves the index page at /
async fn index(req: HttpRequest, hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let username = req.cookie("username").map(|cookie| cookie.value().to_string());

    let mut data = serde_json::Map::new();
    if let Some(username) = username {
        data.insert("username".to_string(), json!(username));
        data.insert("logged_in".to_string(), json!(true));
    } else {
        data.insert("logged_in".to_string(), json!(false));
    }

    let body = hb.render("index", &data).unwrap_or_else(|_| "Template error".to_string());
    HttpResponse::Ok().body(body)
}

// Main function to start the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();

    // Initialize the database pool
    let pool = SqlitePool::connect("sqlite:health_services.db")
        .await
        .expect("Failed to connect to database");

    // Initialize handlebars template engine
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("index", "./templates/index.hbs")
        .expect("Failed to register templates directory");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Share the database pool across handlers
            .app_data(web::Data::new(handlebars.clone())) // Share the handlebars instance
            .route("/api-key", web::get().to(api_key_handler)) // Endpoint to serve the API key
            .route("/services", web::get().to(services_handler)) // Endpoint for health services
            .route("/", web::get().to(index)) // Endpoint for index page
            .service(login) // Endpoint for login page
            .service(register) // Endpoint for register page
            .service(login_handler) // Endpoint for login form submission
            .service(register_handler) // Endpoint for register form submission
            .service(logout) // Endpoint for logout
            .service(fs::Files::new("/static", "./static").show_files_listing()) // Serve static files under /static
            
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
