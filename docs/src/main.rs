mod find_providers;
use find_providers::{geocode_address, find_health_providers, Coordinates};

use actix_files as fs; 
use actix_web::{get, post, web, App, HttpServer, HttpRequest, Responder, HttpResponse, Result};
use actix_web::cookie::{Cookie, CookieBuilder};
use actix_files::NamedFile;
use serde_json::json;
use serde::{Deserialize};

use sqlx::{SqlitePool};
use handlebars::Handlebars;
use std::sync::Mutex;
use std::borrow::Cow;

use handlebars::{Context, Helper, HelperResult, Output, RenderContext};

#[derive(Deserialize)]
struct QueryParams {
    zip: Option<String>,
    lat: Option<f64>,    // Optional latitude
    lng: Option<f64>,    // Optional longitude
    service_type: Option<String>,
}

#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct FavoriteService {
    photo: String,
    name: String,
    address: String,
    rating: String,
}

struct AppState {
    logout_flag: Mutex<bool>, // Tracks logout state
}

// Handler for the `/services` endpoint
async fn services_handler(req: HttpRequest, query: web::Query<QueryParams>) -> impl Responder {
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

    // Request username cookie 
    let mut isLoggedIn = false;
    let username = req.cookie("username").map(|cookie| cookie.value().to_string());
    if let Some(username) = username {
        if !username.is_empty() {
            isLoggedIn = true;
        }
    }

    HttpResponse::Ok().json(json!({
        "coordinates": coordinates,
        "providers": providers,
        "isLoggedIn": isLoggedIn
    }))
}

// Handler for the `/api-key` endpoint
async fn api_key_handler() -> impl Responder {
    let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
        .expect("GOOGLE_MAPS_API_KEY must be set in environment");
    HttpResponse::Ok().body(api_key)
}

#[post("/favorites")]
async fn save_favorites(
    req: HttpRequest,
    favorite: web::Json<FavoriteService>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    println!("Processing save_favorites request...");
    println!("All Cookies: {:?}", req.cookies());

    // Extract user ID from cookies
    let user_id_cookie = req.cookie("user_id").map(|cookie| cookie.value().to_string());
    if let Some(user_id_str) = user_id_cookie {
        match user_id_str.parse::<i64>() {
            Ok(user_id) => {
                // Convert rating to REAL in SQL database
                // let rating = favorite.rating.parse::<f64>().unwrap_or(0.0);
                // Insert the favorite service into the database
                let query_result = sqlx::query!(
                    "INSERT INTO favorites (user_id, photo, title, address, rating) VALUES (?, ?, ?, ?, ?)",
                    user_id,
                    favorite.photo,
                    favorite.name,
                    favorite.address,
                    favorite.rating
                )
                .execute(pool.get_ref())
                .await;

                match query_result {
                    Ok(_) => {
                        println!("Favorite saved successfully.");
                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "message": "Favorite saved successfully."
                        }))
                    }
                    Err(e) => {
                        HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Failed to save favorite. Please try again later."
                        }))
                    }
                }
            }
            Err(_) => {
                println!("Invalid user ID in cookie.");
                HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Invalid user ID."
                }))
            }
        }
    } else {
        println!("User not logged in.");
        HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "User not logged in. Please log in and try again."
        }))
    }
}


// Handler for the `/login` endpoint
#[post("/login")]
async fn login_handler(
    form: web::Form<LoginData>,
    pool: web::Data<SqlitePool>,
    hb: web::Data<Handlebars<'_>>,
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
                let username_cookie = CookieBuilder::new("username", username.clone())
                    .path("/")
                    .finish();

                // Set a cookie with the user ID on successful login
                let user_id_cookie = CookieBuilder::new("user_id", Cow::from(user.id.unwrap().to_string()))
                    .path("/")
                    .finish();

                // Redirect back to the index page
                return HttpResponse::Found()
                    .cookie(username_cookie) // Attach the cookie to the response
                    .cookie(user_id_cookie) // Attach the user ID cookie to the response
                    .append_header(("Location", "/")) // Redirect to the index page
                    .finish();
            } else {
                // If invalid credentials, reroute to login page with handlebars message
                let mut data = serde_json::Map::new();
                data.insert("error_invalid_credentials".to_string(), json!("Invalid credentials. Please try again."));
                let body = hb.render("login", &data).unwrap_or_else(|_| "Template error".to_string());
                return HttpResponse::Ok().body(body);
            }
        }
        Ok(None) => {
            // If user not found, reroute to login page with handlebars message
            let mut data = serde_json::Map::new();
            data.insert("error_not_found".to_string(), json!("User not found. Please register for an account."));
            let body = hb.render("login", &data).unwrap_or_else(|_| "Template error".to_string());
            return HttpResponse::Ok().body(body);
        }
        Err(_) => {
            // If database error, reroute to login page with handlebars message
            let mut data = serde_json::Map::new();
            data.insert("error_database".to_string(), json!("Error retrieving user from database. Please try again later."));
            let body = hb.render("login", &data).unwrap_or_else(|_| "Template error".to_string());
            return HttpResponse::Ok().body(body);
        }
    }
}

// Serves the handlebars login page at /login
#[get("/login")]
async fn login(hb: web::Data<Handlebars<'_>>, _state: web::Data<AppState>) -> impl Responder {
    let data = serde_json::Map::new();
    let body = hb.render("login", &data).unwrap_or_else(|_| "Template error".to_string());
    HttpResponse::Ok().body(body)
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
                Ok(res) if res.rows_affected() == 1 => {
                    // Redirect to the /register page with a success message
                    let success_message = format!("Success!");
                    return HttpResponse::Found()
                        .header("Location", format!("/register?success={}", success_message))
                        .finish();
                }
                _ => HttpResponse::InternalServerError().body("Failed to register user"),
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}
// Serves the register page at /register
// #[get("/register")]
async fn register(req: HttpRequest, hb: web::Data<Handlebars<'_>>, _state: web::Data<AppState>) -> impl Responder {
    let success_message = req.query_string();
    let mut data = serde_json::Map::new();

    if !success_message.is_empty() {
        data.insert("success".to_string(), json!(success_message));
    }
    let body = hb.render("register",&data).unwrap_or_else(|_| "Template error".to_string());
    HttpResponse::Ok().body(body)
}

// Serves the profile page at /profile
// #[get("/profile")]
async fn profile(req: HttpRequest, pool: web::Data<SqlitePool>, hb: web::Data<Handlebars<'_>>, _state: web::Data<AppState>) -> impl Responder {
    // Extract the user ID from the cookies
    let user_id_cookie = req.cookie("user_id").and_then(|cookie| cookie.value().parse::<i64>().ok());
    let mut data = serde_json::Map::new();

    // Add username from cookie 
    if let Some(username) = req.cookie("username").map(|cookie| cookie.value().to_string()) {
        data.insert("username".to_string(), json!(username));
    }

    // Fetch user's favorites from the database using username cookie 
        // Fetch user's favorites if logged in

        if let Some(user_id) = user_id_cookie {

            match sqlx::query!(
    
                "SELECT id, photo, title AS name, address, rating FROM favorites WHERE user_id = ?",
    
                user_id
    
            )
    
            .fetch_all(pool.get_ref())
    
            .await {
    
                Ok(favorites) => {
    
                    let favorites_json: Vec<serde_json::Value> = favorites.into_iter().map(|f| json!({
    
                        "id": f.id,
    
                        "photo": f.photo,
    
                        "name": f.name,
    
                        "address": f.address,
    
                        "rating": f.rating
    
                    })).collect();
    
                    
    
                    data.insert("favorites".to_string(), json!(favorites_json));
    
                }
    
                Err(_) => {
    
                    // Handle database error
    
                    data.insert("error".to_string(), json!("Could not fetch favorites"));
    
                }
    
            }
    
        }
    
        let body = hb.render("profile", &data).unwrap_or_else(|_| "Template error".to_string());
    
        HttpResponse::Ok().body(body)
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

    // Clear the user ID cookie by setting it to an empty value
    let user_id_cookie = Cookie::build("user_id", "")
        .path("/")
        .finish();
    
    // Respond with a redirect to the index page
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .cookie(cookie) // Include the cleared cookie to remove the username
        .cookie(user_id_cookie) // Include the cleared user ID cookie
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
        .expect("Failed to register profile");

    handlebars.register_template_file("register", "./templates/register.hbs")
        .expect("Failed to register register");

    handlebars.register_template_file("login", "./templates/login.hbs")
        .expect("Failed to register login");

    handlebars.register_helper("times", Box::new(|h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
        let param = h.param(0).unwrap();
        let times = param.value().as_f64().unwrap_or(0.0) as usize;
        for _ in 0..times {
            out.write("*")?;
        }
        Ok(())
    }));

    handlebars.register_helper("subtract", Box::new(|h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
        let first = h.param(0).unwrap().value().as_f64().unwrap_or(0.0);
        let second = h.param(1).unwrap().value().as_f64().unwrap_or(0.0);
        out.write(&(first - second).to_string())?;
        Ok(())
    }));

    // Create the application state
    let state = web::Data::new(AppState {
        logout_flag: Mutex::new(false),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Share the database pool across handlers
            .app_data(web::Data::new(handlebars.clone())) // Share the handlebars instance
            .app_data(state.clone()) // Share the application state
            .app_data(web::JsonConfig::default())
            .route("/api-key", web::get().to(api_key_handler)) // Endpoint to serve the API key
            .route("/services", web::get().to(services_handler)) // Endpoint for health services
            .route("/", web::get().to(index)) // Endpoint for index page
            .route("/profile", web::get().to(profile)) // Endpoint for profile page
            .route("/register", web::get().to(register)) // Endpoint for register page
            .service(save_favorites) // Endpoint for saving favorites
            .service(login) // Endpoint for login page
            .service(login_handler) // Endpoint for login form submission
            .service(register_handler) // Endpoint for register form submission
            .service(logout) // Endpoint for logout
            .service(fs::Files::new("/static", "./static").show_files_listing()) // Serve static files under /static
            
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
