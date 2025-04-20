use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use mongodb::{Database, Collection};
use mongodb::bson::{doc, Document};
use base64::{Engine, engine::general_purpose};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};

#[derive(Deserialize, Serialize, Clone)]
pub struct ProblemResponse {
    pub tags: Vec<String>,
    pub image: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct QBankEntry {
    pub id: String,
    pub tags: Vec<String>,
    pub path: String,
}

#[get("/get/{problem_id}")]
async fn get_problem(
    db: web::Data<Database>,
    _user: ClaimsValidator,
    path: web::Path<String>,
) -> ApiResult<impl Responder> {
    let problem_id = path.into_inner();
    
    let collection: Collection<Document> = db.collection("qbank");
    let problem = collection
        .find_one(doc! { "_id": &problem_id })
        .await?;
    
    match problem {
        Some(doc) => {
            let tags = match doc.get_array("tags") {
                Ok(tags_array) => {
                    tags_array
                        .iter()
                        .filter_map(|tag| tag.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                }
                Err(_) => Vec::new(),
            };
            
            let image = match doc.get_binary_generic("image") {
                Ok(binary) => {
                    // Use standard library base64 encoding
                    let base64_string = general_purpose::STANDARD.encode(binary);
                    base64_string
                }
                Err(_) => {
                    return Err(ApiError::new(
                        ApiErrorType::Internal,
                        "Failed to extract image data".to_string(),
                    ))
                }
            };
            
            Ok(HttpResponse::Ok().json(ProblemResponse { tags, image }))
        }
        None => Err(ApiError::new(
            ApiErrorType::NotFound,
            format!("Problem with ID {} not found", problem_id),
        )),
    }
}

#[get("/qbank")]
async fn get_qbank(
    qbank_data: web::Data<Vec<QBankEntry>>,
    _user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok().json(qbank_data.get_ref()))
}

pub fn api_scope() -> Scope {
    web::scope("/problem")
        .service(get_problem)
        .service(get_qbank)
}