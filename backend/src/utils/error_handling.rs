use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;
use log::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal Server Error")]
    InternalError,
    #[error("Bad Request")]
    BadRequest,
    #[error("Unauthorized")]
    Unauthorized,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AppError::InternalError => {
                error!("Internal Server Error: {:?}", self);
                HttpResponse::InternalServerError().json("Internal Server Error")
            }
            AppError::BadRequest => HttpResponse::BadRequest().json("Bad Request"),
            AppError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
        }
    }
}
