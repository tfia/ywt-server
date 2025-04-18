use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use mongodb::Database;
use mongodb::bson::{doc, Document};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::db::{check_user_exists, check_admin_exists};
use crate::api::stats::StatsResponse as GetUserStatsResponse;

#[derive(Serialize)]
pub struct GetUserListResponse {
    pub usernames: Vec<String>,
    pub emails: Vec<String>,
    pub created_at: Vec<String>,
}

#[derive(Deserialize)]
pub struct DeleteUserRequest {
    pub username: String,
}

#[derive(Deserialize)]
pub struct GetUserStatsRequest {
    pub username: String,
}

#[get("/list")]
async fn get_user_list(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    // Verify if the user is an admin
    if !check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Admin access required".to_string(),
        ));
    }

    // Get the list of users from the database
    let collection = db.collection::<Document>("users");
    let mut cursor = collection.find(doc! {}).await?;
    let mut usernames = Vec::new();
    let mut emails = Vec::new();
    let mut created_at = Vec::new();

    while let Some(user_doc) = cursor.try_next().await? {
        if let Some(username) = user_doc.get_str("username").ok() {
            usernames.push(username.to_string());
        }
        if let Some(email) = user_doc.get_str("email").ok() {
            emails.push(email.to_string());
        }
        if let Some(created_at_str) = user_doc.get_str("created_at").ok() {
            created_at.push(created_at_str.to_string());
        }
    }

    Ok(HttpResponse::Ok().json(GetUserListResponse { usernames, emails, created_at }))
}

#[post("/delete")]
async fn delete_user(
    db: web::Data<Database>,
    user: ClaimsValidator,
    req: web::Json<DeleteUserRequest>,
) -> ApiResult<impl Responder> {
    // Verify if the user is an admin
    if !check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Admin access required".to_string(),
        ));
    }

    if !check_user_exists(&db, &req.username).await? {
        return Err(ApiError::new_not_found());
    }

    let collection = db.collection::<Document>("users");
    collection.delete_one(doc! { "username": &req.username }).await?;

    // also delete the user's stats
    let collection = db.collection::<Document>("stats");
    collection.delete_one(doc! { "username": &req.username }).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

#[get("/stats")]
async fn get_user_stats(
    db: web::Data<Database>,
    user: ClaimsValidator,
    req: web::Json<GetUserStatsRequest>,
) -> ApiResult<impl Responder> {
    // Verify if the user is an admin
    if !check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Admin access required".to_string(),
        ));
    }

    if !check_user_exists(&db, &req.username).await? {
        return Err(ApiError::new_not_found());
    }

    let collection = db.collection::<Document>("stats");
    let user_doc = collection
        .find_one(doc! { "username": &req.username })
        .await?;

    match user_doc {
        Some(doc) => {
            let conversation = doc.get_i32("conversation")?;
            let tags_doc = doc.get_document("tags")?;
            let mut tags: Vec<(String, i32)> = vec![];
            for (key, value) in tags_doc.iter() {
                if let Some(count) = value.as_i32() {
                    tags.push((key.to_string(), count));
                }
            }
            Ok(HttpResponse::Ok().json(GetUserStatsResponse { conversation, tags }))
        }
        None => Err(ApiError::new_not_found()),
    }
}

pub fn api_scope() -> Scope {
    web::scope("/users")
        .service(get_user_list)
        .service(delete_user)
        .service(get_user_stats)
}
