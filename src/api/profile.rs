use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use mongodb::bson::{doc, Document};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::db::{
    check_user_exists,
    check_admin_exists,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct ProfileResponse {
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[get("")]
async fn profile(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {  
    let username = user.username;
    let collection = match check_user_exists(&db, &username).await {
        Ok(true) => db.collection::<Document>("users"),
        Ok(false) => match check_admin_exists(&db, &username).await {
            Ok(true) => db.collection::<Document>("admins"),
            Ok(false) => return Err(ApiError::new(ApiErrorType::InvalidRequest, "User not found".to_string())),
            Err(_) => return Err(ApiError::new(ApiErrorType::Internal, "Database error".to_string())),
        },
        Err(_) => return Err(ApiError::new(ApiErrorType::Internal, "Database error".to_string())),
    };
    let user: Document = collection
        .find_one(doc! { "username": &username })
        .await?
        .ok_or(ApiError::new(
            ApiErrorType::InvalidRequest,
            "User not found".to_string(),
        ))?;
    
    let email = user.get_str("email")?.to_string();
    let created_at = user.get_str("created_at")?.to_string();

    Ok(HttpResponse::Ok().json(ProfileResponse { username, created_at, email }))
}

pub fn api_scope() -> Scope {
    web::scope("/profile").service(profile)
}