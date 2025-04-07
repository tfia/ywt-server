use actix_web::{post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use mongodb::bson::doc;

use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::jwt::ClaimsValidator;
use crate::db::{
    check_user_exists,
    check_admin_exists,
    create_user,
    AdminType, UserType
};

const MAX_USERNAME: usize = 32;
const MAX_EMAIL: usize = 64;
const MAX_PASSWORD: usize = 64;
const MIN_PASSWORD: usize = 8;

#[derive(Deserialize, Serialize, Clone)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RegisterResponse {
    pub created_at: String,
}

fn check_req(req: &RegisterRequest) -> ApiResult<()> {
    if req.username.len() > MAX_USERNAME || req.username.len() == 0 {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid username".to_string(),
        ));
    }
    if req.email.len() > MAX_EMAIL || req.email.len() == 0 {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid email".to_string(),
        ));
    }
    if req.password.len() > MAX_PASSWORD || req.password.len() < MIN_PASSWORD {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid password".to_string(),
        ));
    }
    Ok(())
}

#[post("")]
async fn register(
    db: web::Data<Database>,
    req: web::Json<RegisterRequest>,
) -> ApiResult<impl Responder> {
    check_req(&req)?;

    if check_admin_exists(&db, &req.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Username already exists".to_string(),
        ));
    }
    
    if check_user_exists(&db, &req.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Username already exists".to_string(),
        ));
    }

    let created_at = chrono::Local::now().to_string();
    create_user::<UserType>(&db, &req.username, &req.email, &req.password, &created_at).await?;

    let collection = db.collection("stats");
    let tag_doc = doc! {
        "username": &req.username,
        "conversation": 0,
        "tags": {}, // Initialize with an empty object
    };
    collection.insert_one(tag_doc).await?;

    Ok(HttpResponse::Ok().json(RegisterResponse { created_at }))
}

#[post("/admin")]
async fn admin_register(
    db: web::Data<Database>,
    req: web::Json<RegisterRequest>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    // check if user is admin
    if !check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }
    
    check_req(&req)?;

    // check if user with the same username exists
    if check_user_exists(&db, &req.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }

    if check_admin_exists(&db, &req.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }

    let created_at = chrono::Local::now().to_string();
    create_user::<AdminType>(&db, &req.username, &req.email, &req.password, &created_at).await?;

    Ok(HttpResponse::Ok().json(RegisterResponse { created_at }))
}

pub fn api_scope() -> Scope {
    web::scope("/register")
        .service(register)
        .service(admin_register)
}