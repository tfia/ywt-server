use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::{Database, Collection};
use mongodb::bson::{doc, Document};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::db;

#[derive(Deserialize, Serialize, Clone)]
pub struct StatsRequest {
    pub tag: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StatsResponse {
    pub conversation: i32,
    pub tags: Vec<(String, i32)>,
}

#[post("")]
async fn post_stats(
    db: web::Data<Database>,
    user: ClaimsValidator,
    req: web::Json<StatsRequest>,
) -> ApiResult<impl Responder> {
    let collection: Collection<Document> = db.collection("stats");
    let tags = &req.tag;
    let mut update_doc = doc! {};
    for tag in tags {
        update_doc.insert(
            format!("tags.{}", tag),
            1,
        );
    }
    collection
        .update_one(
            doc! { "username": &user.username },
            doc! { "$inc": update_doc },
        )
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

#[post("/conv")]
async fn post_conv_stats(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    let collection: Collection<Document> = db.collection("stats");
    collection
        .update_one(
            doc! { "username": &user.username },
            doc! { "$inc": { "conversation": 1 } },
        )
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

#[get("")]
async fn get_stats(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    let collection: Collection<Document> = db.collection("stats");
    let user_doc = collection
        .find_one(doc! { "username": &user.username })
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
            Ok(HttpResponse::Ok().json(StatsResponse {
                conversation,
                tags,
            }))
        }
        None => Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "User not found".to_string(),
        )),
    }
}

#[post("/clear")]
async fn clear_stats(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    // Verify if the user is an admin
    if !db::check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Admin access required".to_string(),
        ));
    }

    // clear all stats
    let collection: Collection<Document> = db.collection("stats");
    collection
        .update_many(
            doc! {},
            doc! { "$set": { "conversation": 0, "tags": {} } },
        )
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

pub fn api_scope() -> Scope {
    web::scope("/stats")
        .service(post_stats)
        .service(get_stats)
        .service(post_conv_stats)
        .service(clear_stats)
}