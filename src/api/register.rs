use actix_web::{post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::{Collection, Database};
use mongodb::bson::{doc, Document};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};

use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::jwt::ClaimsValidator;

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

    // check if admin with the same username exists
    let collection: Collection<Document> = db.collection("admins");
    let existing_admin = collection
        .find_one(doc! { "username": &req.username })
        .await?;
    if existing_admin.is_some() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Username already exists".to_string(),
        ));
    }
    
    let collection = db.collection("users");
    let existing_user = collection
        .find_one(doc! { "username": &req.username })
        .await?;
    if existing_user.is_some() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Username already exists".to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)?.to_string();
    let created_at = chrono::Local::now().to_string();
    let user_doc = doc! {
        "username": &req.username,
        "email": &req.email,
        "password": password_hash,
        "created_at": &created_at,
    };
    collection.insert_one(user_doc).await?;

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
    let collection: Collection<Document> = db.collection("admins");
    if collection
        .find_one(doc! { "username": &user.username })
        .await?.is_none() {
            return Err(ApiError::new(
                ApiErrorType::InvalidRequest,
                "Error".to_string(),
            ));
        }
    
    check_req(&req)?;

    // check if user with the same username exists
    let collection: Collection<Document> = db.collection("users");
    let existing_user = collection
        .find_one(doc! { "username": &req.username })
        .await?;
    if existing_user.is_some() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }

    let collection = db.collection("admins");
    let existing_user = collection
        .find_one(doc! { "username": &req.username })
        .await?;
    if existing_user.is_some() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)?.to_string();
    let created_at = chrono::Local::now().to_string();
    let user_doc = doc! {
        "username": &req.username,
        "email": &req.email,
        "password": password_hash,
        "created_at": &created_at,
    };
    collection.insert_one(user_doc).await?;

    Ok(HttpResponse::Ok().json(RegisterResponse { created_at }))
}

pub fn api_scope() -> Scope {
    web::scope("/register")
        .service(register)
        .service(admin_register)
}