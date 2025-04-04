use actix_web::{post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use mongodb::bson::{doc, Document};
use argon2::{
    password_hash::{
        PasswordHash, PasswordVerifier
    },
    Argon2
};

use crate::jwt;
use crate::error::{ApiResult, ApiError, ApiErrorType};

#[derive(Deserialize, Serialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LoginResponse {
    pub token: String,
}

#[post("")]
async fn login(
    db: web::Data<Database>,
    req: web::Json<LoginRequest>,
) -> ApiResult<impl Responder> {  
    let collection = db.collection("users");
    let user: Document = collection
        .find_one(doc! { "username": &req.username })
        .await?
        .ok_or(ApiError::new(
            ApiErrorType::InvalidRequest,
            "User not found".to_string(),
        ))?;
    
    let password: &str = user.get_str("password")?;
    let parsed_hash = PasswordHash::new(password)?;
    if Argon2::default().verify_password(&req.password.as_bytes(), &parsed_hash).is_err() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid password".to_string(),
        ));
    }

    let token = jwt::Claims::create_jwt(req.username.clone(), 12)?;

    Ok(HttpResponse::Ok().json(LoginResponse { token }))
}

#[post("/admin")]
async fn admin_login(
    db: web::Data<Database>,
    req: web::Json<LoginRequest>,
) -> ApiResult<impl Responder> {
    let collection = db.collection("admins");
    let user: Document = collection
        .find_one(doc! { "username": &req.username })
        .await?
        .ok_or(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ))?;
    
    let password: &str = user.get_str("password")?;
    let parsed_hash = PasswordHash::new(password)?;
    if Argon2::default().verify_password(&req.password.as_bytes(), &parsed_hash).is_err() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }

    let token = jwt::Claims::create_jwt(req.username.clone(), 12)?;

    Ok(HttpResponse::Ok().json(LoginResponse { token }))
}

pub fn api_scope() -> Scope {
    web::scope("/login")
        .service(login)
        .service(admin_login)
}