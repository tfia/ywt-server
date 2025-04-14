use actix_web::{post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use mongodb::bson::{doc, Document};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordVerifier, SaltString, PasswordHasher
    },
    Argon2
};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::db::{check_user_exists, check_admin_exists};
use crate::utils::{check_username, check_password};

#[derive(Deserialize)]
pub struct ModifyUsernameRequest {
    pub new_username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ModifyPasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ModifyResponse {
    pub status: String,
}

#[post("/username")]
async fn modify_username(
    db: web::Data<Database>,
    user: ClaimsValidator,
    req: web::Json<ModifyUsernameRequest>,
) -> ApiResult<impl Responder> {  
    check_username(&req.new_username)?;

    // Check if the new username already exists
    if check_user_exists(&db, &req.new_username).await? || check_admin_exists(&db, &req.new_username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Username already exists".to_string(),
        ));
    }
    
    // Find the current user
    let collection = db.collection::<Document>("users");
    let user_doc = collection
        .find_one(doc! { "username": &user.username })
        .await?
        .ok_or(ApiError::new(
            ApiErrorType::InvalidRequest,
            "User not found".to_string(),
        ))?;
    
    // Verify the password
    let stored_password = user_doc.get_str("password")?;
    let parsed_hash = PasswordHash::new(stored_password)?;
    if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_err() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid password".to_string(),
        ));
    }
    
    // Update the username
    collection
        .update_one(
            doc! { "username": &user.username },
            doc! { "$set": { "username": &req.new_username } },
        )
        .await?;
    
    Ok(HttpResponse::Ok().json(ModifyResponse { 
        status: "success".to_string() 
    }))
}

#[post("/password")]
async fn modify_password(
    db: web::Data<Database>,
    user: ClaimsValidator,
    req: web::Json<ModifyPasswordRequest>,
) -> ApiResult<impl Responder> {  
    check_password(&req.new_password)?;

    let collection = db.collection::<Document>("users");
    
    // Find the current user
    let user_doc = collection
        .find_one(doc! { "username": &user.username })
        .await?
        .ok_or(ApiError::new(
            ApiErrorType::InvalidRequest,
            "User not found".to_string(),
        ))?;
    
    // Verify the current password
    let stored_password = user_doc.get_str("password")?;
    let parsed_hash = PasswordHash::new(stored_password)?;
    if Argon2::default().verify_password(req.current_password.as_bytes(), &parsed_hash).is_err() {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid current password".to_string(),
        ));
    }
    
    // Hash the new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.new_password.as_bytes(), &salt)?
        .to_string();
    
    // Update the password
    collection
        .update_one(
            doc! { "username": &user.username },
            doc! { "$set": { "password": password_hash } },
        )
        .await?;
    
    Ok(HttpResponse::Ok().json(ModifyResponse { 
        status: "success".to_string() 
    }))
}

#[post("/delete")]
async fn delete_account(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {  
    let collection = db.collection::<Document>("users");

    collection
        .delete_one(doc! { "username": &user.username })
        .await?;
    
    Ok(HttpResponse::Ok().json(ModifyResponse { 
        status: "success".to_string() 
    }))
}

pub fn api_scope() -> Scope {
    web::scope("/modify")
        .service(modify_username)
        .service(modify_password)
        .service(delete_account)
}
