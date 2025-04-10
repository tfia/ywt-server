use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::Deserialize;
use mongodb::{Collection, Database};
use mongodb::bson::{doc, Document};

use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::db::activate_user;

#[derive(Deserialize)]
pub struct ActivationRequest {
    code: String,
}

#[get("/{username}")]
async fn verify_email(
    db: web::Data<Database>,
    path: web::Path<String>,
    query: web::Query<ActivationRequest>,
) -> ApiResult<impl Responder> {
    let username = path.into_inner();
    let code = &query.code;
    
    // Verify the activation code
    let activation_collection: Collection<Document> = db.collection("activation_codes");
    let filter = doc! {
        "username": &username,
        "code": code,
    };
    
    let activation_doc = activation_collection.find_one(filter.clone()).await?;
    
    if let Some(doc) = activation_doc {
        // Check if the activation code has expired
        let expires_at = doc.get_str("expires_at")?;
        let expires_datetime = chrono::DateTime::parse_from_str(expires_at, "%Y-%m-%d %H:%M:%S%.f %z")
            .map_err(|_| ApiError::new(
                ApiErrorType::Internal,
                "Failed to parse expiration date".to_string(),
            ))?;
            
        if chrono::Local::now() > expires_datetime {
            return Err(ApiError::new(
                ApiErrorType::InvalidRequest,
                "Activation code has expired".to_string(),
            ));
        }
        
        // Activate the user
        activate_user(&db, &username).await?;
        
        // Remove the activation code
        activation_collection.delete_one(filter).await?;
        
        Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
    } else {
        Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid activation code".to_string(),
        ))
    }
}

pub fn api_scope() -> Scope {
    web::scope("/verify_email")
        .service(verify_email)
}
