use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub specialty: String,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Appointment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider_id: Uuid,
    pub appointment_time: NaiveDateTime,
    pub is_available: bool,
}
