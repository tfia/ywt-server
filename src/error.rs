use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ApiErrorType {
    NotFound = 1,
    InvalidRequest = 2,
    Internal = 3,
}

impl ApiErrorType {
    fn reason(&self) -> &'static str {
        match self {
            ApiErrorType::NotFound => "ERR_NOT_FOUND",
            ApiErrorType::InvalidRequest => "ERR_INVALID_REQUEST",
            ApiErrorType::Internal => "ERR_INTERNAL_SERVER_ERROR",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiErrorType::NotFound => StatusCode::NOT_FOUND,
            ApiErrorType::InvalidRequest => StatusCode::BAD_REQUEST,
            ApiErrorType::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug)]
pub struct ApiError {
    error_type: ApiErrorType,
    message: String,
}

impl ApiError {
    pub fn new(error_type: ApiErrorType, message: String) -> Self {
        ApiError { error_type, message }
    }
    pub fn new_not_found() -> Self {
        ApiError::new(
            ApiErrorType::NotFound, 
            "Resource not found".to_string(),
        )
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u8,
    reason: &'static str,
    message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_type.reason())
    }
}

impl std::error::Error for ApiError {}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.error_type.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            log::error!("Internal server error: {}", self.message);
        }
        HttpResponse::build(status_code).json(ErrorResponse {
            code: self.error_type as u8,
            reason: self.error_type.reason(),
            message: self.message.clone(),
        })
    }
}

impl From<mongodb::error::Error> for ApiError {
    fn from(err: mongodb::error::Error) -> Self {
        ApiError::new(
            ApiErrorType::Internal,
            format!("MongoDB error: {}", err),
        )
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(err: argon2::password_hash::Error) -> Self {
        ApiError::new(
            ApiErrorType::Internal,
            format!("Argon2 error: {}", err),
        )
    }
}

pub type ApiResult<T> = Result<T, ApiError>;