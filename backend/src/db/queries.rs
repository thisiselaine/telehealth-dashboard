use crate::db::models::{User, Provider, Appointment};
use sqlx::Mssql;
use uuid::Uuid;
use chrono::{NaiveDateTime};

// User CRUD Operations
pub async fn create_user(pool: &sqlx::Pool<Mssql>, user: User) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "INSERT INTO users (id, username, email, hashed_password) VALUES ($1, $2, $3, $4) RETURNING *",
        Uuid::new_v4(),
        user.username,
        user.email,
        user.hashed_password
    )
    .fetch_one(pool)
    .await
}

pub async fn get_user_by_email(pool: &sqlx::Pool<Mssql>, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await
}

// Provider CRUD Operations
pub async fn create_provider(pool: &sqlx::Pool<Mssql>, provider: Provider) -> Result<Provider, sqlx::Error> {
    sqlx::query_as!(
        Provider,
        "INSERT INTO providers (id, name, specialty, location) VALUES ($1, $2, $3, $4) RETURNING *",
        Uuid::new_v4(),
        provider.name,
        provider.specialty,
        provider.location
    )
    .fetch_one(pool)
    .await
}

pub async fn get_providers_by_specialty(pool: &sqlx::Pool<Mssql>, specialty: &str) -> Result<Vec<Provider>, sqlx::Error> {
    sqlx::query_as!(
        Provider,
        "SELECT * FROM providers WHERE specialty = $1",
        specialty
    )
    .fetch_all(pool)
    .await
}

// Appointment CRUD Operations
pub async fn create_appointment(pool: &sqlx::Pool<Mssql>, appointment: Appointment) -> Result<Appointment, sqlx::Error> {
    sqlx::query_as!(
        Appointment,
        "INSERT INTO appointments (id, user_id, provider_id, appointment_time, is_available) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        Uuid::new_v4(),
        appointment.user_id,
        appointment.provider_id,
        appointment.appointment_time,
        appointment.is_available
    )
    .fetch_one(pool)
    .await
}

pub async fn get_appointments_by_provider(pool: &sqlx::Pool<Mssql>, provider_id: Uuid) -> Result<Vec<Appointment>, sqlx::Error> {
    sqlx::query_as!(
        Appointment,
        "SELECT * FROM appointments WHERE provider_id = $1",
        provider_id
    )
    .fetch_all(pool)
    .await
}
