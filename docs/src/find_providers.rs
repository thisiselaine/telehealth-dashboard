use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthProvider {
    name: String,
    address: String,
    distance: f64,
    provider_type: String,
    phone: Option<String>,
    rating: Option<f32>,
    photo_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lng: f64,
}

pub async fn geocode_address(address: &str, api_key: &str) -> Result<Coordinates, Box<dyn Error>> {
    let encoded_address = urlencoding::encode(address);
    let url = format!(
        "https://maps.googleapis.com/maps/api/geocode/json?address={}&key={}",
        encoded_address, api_key
    );

    let response: serde_json::Value = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    let location = &response["results"][0]["geometry"]["location"];
    
    Ok(Coordinates {
        lat: location["lat"].as_f64().unwrap(),
        lng: location["lng"].as_f64().unwrap(),
    })
}

pub async fn find_health_providers(
    coordinates: &Coordinates,
    radius_meters: u32,
    api_key: &str,
    service_type: &str,
) -> Result<Vec<HealthProvider>, Box<dyn Error>> {
    let url = format!(
        "https://maps.googleapis.com/maps/api/place/nearbysearch/json?location={},{}&radius={}&type={}&key={}",
        coordinates.lat, coordinates.lng, radius_meters, service_type, api_key
    );
    // println!("{}", url);

    let response: serde_json::Value = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    let mut providers = Vec::new();

    if let Some(results) = response["results"].as_array() {
        for result in results {

            let photo_url = result["photos"]
                .as_array()
                .and_then(|photos| photos.get(0)) // Get the first photo reference
                .and_then(|photo| photo["photo_reference"].as_str())
                .map(|photo_reference| {
                    format!(
                        "https://maps.googleapis.com/maps/api/place/photo?maxwidth=400&photoreference={}&key={}",
                        photo_reference, api_key
                    )
                });

            let provider = HealthProvider {
                name: result["name"].as_str().unwrap_or("").to_string(),
                address: result["vicinity"].as_str().unwrap_or("").to_string(),
                distance: calculate_distance(
                    coordinates,
                    result["geometry"]["location"]["lat"].as_f64().unwrap(),
                    result["geometry"]["location"]["lng"].as_f64().unwrap(),
                ),
                provider_type: result["types"][0].as_str().unwrap_or("").to_string(),
                phone: result["formatted_phone_number"]
                    .as_str()
                    .map(String::from),
                rating: result["rating"].as_f64().map(|r| r as f32),
                photo_url,
            };
            providers.push(provider);
        }
    }

    Ok(providers)
}

fn calculate_distance(coords: &Coordinates, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS: f64 = 6371.0; // kilometers

    let lat1_rad = coords.lat.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - coords.lat).to_radians();
    let delta_lng = (lng2 - coords.lng).to_radians();

    let a = (delta_lat / 2.0).sin() * (delta_lat / 2.0).sin()
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin() * (delta_lng / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c
}