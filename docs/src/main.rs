mod find_providers;
use find_providers::{geocode_address, find_health_providers, Coordinates};

use actix_files as fs; 
use actix_web::{get, post, web, App, HttpServer, HttpRequest, Responder, HttpResponse, Result};
use actix_web::cookie::{Cookie, CookieBuilder};
use actix_files::NamedFile;
use serde_json::json;
use serde::{Deserialize};

use sqlx::SqlitePool;
use handlebars::Handlebars;
use std::sync::Mutex;

#[derive(Deserialize)]
struct QueryParams {
    zip: Option<String>,
    lat: Option<f64>, // Optional latitude
    lng: Option<f64>, // Optional longitude
    service_type: Option<String>,
}

#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

struct AppState {
    logout_flag: Mutex<bool>, // Tracks logout state
}

// Handler for the `/services` endpoint
async fn services_handler(query: web::Query<QueryParams>) -> impl Responder {
    let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
        .expect("GOOGLE_MAPS_API_KEY must be set in environment");

    let coordinates = if let (Some(lat), Some(lng)) = (query.lat, query.lng) {
        // Use lat/lng if provided
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

    // Default to a generic service type if none is specified
    let service_type = query.service_type.as_deref().unwrap_or("hospital");

    let providers = match find_health_providers(&coordinates, 10000, &api_key, service_type).await {
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

                // Redirect back to the index page
                return HttpResponse::Found()
                    .cookie(cookie) // Attach the cookie to the response
                    .append_header(("Location", "/")) // Redirect to the index page
                    .finish();
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

// Serves the profile page at /profile
#[get("/profile")]
async fn profile(_req: HttpRequest) -> Result<NamedFile> {
    NamedFile::open_async("./templates/profile.hbs").await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("File open error: {}", e))
    })
}

// Handler for the `/logout` endpoint
#[post("/logout")]
async fn logout(state: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    // Set the logout flag in the shared state
    {
        let mut logout_flag = state.logout_flag.lock().unwrap();
        *logout_flag = true;
    }

    // Clear the username cookie by setting it to an empty value
    let cookie = Cookie::build("username", "")
        .path("/")
        .finish();
    
    // Respond with a redirect to the index page
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .cookie(cookie) // Include the cleared cookie to remove the username
        .finish()
}

// Serves the index page at /
// #[get("/")]
async fn index(req: HttpRequest, hb: web::Data<Handlebars<'_>>, state: web::Data<AppState>) -> impl Responder {
    let username = req.cookie("username").map(|cookie| cookie.value().to_string());

    let mut data = serde_json::Map::new();
    if let Some(username) = username {
        if !username.is_empty() {
            data.insert("username".to_string(), json!(username));
            // Debug 
            println!("User {} is logged in", username);

            data.insert("logged_in".to_string(), json!(true));
        } else {
            data.insert("logged_in".to_string(), json!(false));
        }
    } else {
        data.insert("logged_in".to_string(), json!(false));
    }

    // Check if the logout flag is set
    {
        let mut logout_flag = state.logout_flag.lock().unwrap();
        if *logout_flag {
            data.insert("logout_message".to_string(), json!("You have successfully logged out."));
            *logout_flag = false; // Reset the flag after showing the message
        }
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
        .expect("Failed to register index");

    handlebars.register_template_file("profile", "./templates/profile.hbs")
        .expect("Failed to register templates index");

    // Create the application state
    let state = web::Data::new(AppState {
        logout_flag: Mutex::new(false),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Share the database pool across handlers
            .app_data(web::Data::new(handlebars.clone())) // Share the handlebars instance
            .app_data(state.clone()) // Share the application state
            .route("/api-key", web::get().to(api_key_handler)) // Endpoint to serve the API key
            .route("/services", web::get().to(services_handler)) // Endpoint for health services
            .route("/", web::get().to(index)) // Endpoint for index page
            .service(login) // Endpoint for login page
            .service(register) // Endpoint for register page
            .service(login_handler) // Endpoint for login form submission
            .service(register_handler) // Endpoint for register form submission
            .service(logout) // Endpoint for logout
            .service(profile) // Endpoint for profile
            .service(fs::Files::new("/static", "./static").show_files_listing()) // Serve static files under /static
            
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
