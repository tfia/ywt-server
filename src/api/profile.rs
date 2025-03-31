use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use mongodb::bson::{doc, Document};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};

#[derive(Deserialize, Serialize, Clone)]
pub struct ProfileResponse {
    pub created_at: String,
}

#[get("")]
async fn profile(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {  
    let collection = db.collection("users");
    let user: Document = collection
        .find_one(doc! { "username": &user.username })
        .await?
        .ok_or(ApiError::new(
            ApiErrorType::InvalidRequest,
            "User not found".to_string(),
        ))?;
    
    let created_at = user.get_str("created_at")?.to_string();

    Ok(HttpResponse::Ok().json(ProfileResponse { created_at }))
}

pub fn api_scope() -> Scope {
    web::scope("/profile").service(profile)
}