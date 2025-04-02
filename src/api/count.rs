use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::{Database, Collection};
use mongodb::bson::{doc, Document};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};

#[derive(Deserialize, Serialize, Clone)]
pub struct CountRequest {
    pub tag: Vec<String>,
}

#[post("")]
async fn post_count(
    db: web::Data<Database>,
    user: ClaimsValidator,
    req: web::Json<CountRequest>,
) -> ApiResult<impl Responder> {
    let collection: Collection<Document> = db.collection("tags");
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

#[get("")]
async fn get_count(
    db: web::Data<Database>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    let collection: Collection<Document> = db.collection("tags");
    let user_doc = collection
        .find_one(doc! { "username": &user.username })
        .await?;

    match user_doc {
        Some(doc) => {
            let tags = doc.get_document("tags")?;
            Ok(HttpResponse::Ok().json(tags))
        }
        None => Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "User not found".to_string(),
        )),
    }
}

pub fn api_scope() -> Scope {
    web::scope("/count").service(post_count)
        .service(get_count)
}