use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use lettre::message::header::ContentType;
use mongodb::{Database, Collection};
use mongodb::bson::{doc, Document};
use futures::TryStreamExt;
use lettre::{Message, SmtpTransport, Transport};
use serde::Deserialize;

use crate::config::Config;
use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::db;

#[derive(Deserialize, Clone)]
pub struct SendSingleEmailRequest {
    pub username: String,
    pub title: String,
    pub content: String,
}

#[get("")]
async fn send_email(
    db: web::Data<Database>,
    mailer: web::Data<SmtpTransport>,
    user: ClaimsValidator,
    config: web::Data<Config>,
) -> ApiResult<impl Responder> {
    // Verify if the user is an admin
    if !db::check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Admin access required".to_string(),
        ));
    }

    // Get all users and their stats
    let users_collection: Collection<Document> = db.collection("users");
    let mut users_cursor = users_collection.find(doc! {}).await?;

    let stats_collection: Collection<Document> = db.collection("stats");
    while let Some(user_doc) = users_cursor.try_next().await? {
        let email = user_doc.get_str("email")?;
        let username = user_doc.get_str("username")?;

        let sender = format!("YWT Bot <{}>", config.smtp_username);
        let to = format!("{} <{}>", username, email);

        if let Some(stats_doc) = stats_collection.find_one(doc! { "username": username }).await? {
            let tags = stats_doc.get_document("tags")?;
            let tags = tags.keys()
                .map(|k| k.as_str())
                .collect::<Vec<_>>();
            let tag_str = tags.join(", ");
            let conversation_count = stats_doc.get_i32("conversation")?;
            let email = Message::builder()
                .from(sender.parse().unwrap())
                .to(to.parse().unwrap())
                .subject("YWT 答疑周报")
                .header(ContentType::TEXT_PLAIN)
                .body(format!("{} 同学你好！\n\n感谢使用 YWT。以下是你的答疑周报：\n\n在过去一周内，你一共与智能助手交谈 {} 轮次，主要围绕 {} 等知识点。\n\n祝好！\nYWT Team", 
                    username, conversation_count, tag_str))
                .unwrap();

            match mailer.send(&email) {
                Ok(_) => log::info!("Email sent to {}", username),
                Err(e) => {
                    log::error!("Failed to send email to {}: {}", username, e);
                    return Err(ApiError::new(
                        ApiErrorType::Internal,
                        format!("Failed to send email to {}: {}", username, e),
                    ));
                },
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

#[post("/single")]
async fn send_single_email(
    db: web::Data<Database>,
    mailer: web::Data<SmtpTransport>,
    user: ClaimsValidator,
    config: web::Data<Config>,
    req: web::Json<SendSingleEmailRequest>,
) -> ApiResult<impl Responder> {
    // Verify if the user is an admin
    if !db::check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Admin access required".to_string(),
        ));
    }
    let collection: Collection<Document> = db.collection("admins");
    let admin_email = collection
        .find_one(doc! { "username": &user.username })
        .await?
        .ok_or_else(|| ApiError::new_not_found())?
        .get_str("email")?.to_string();

    let collection: Collection<Document> = db.collection("users");
    if let Some(user_doc) = collection.find_one(doc! { "username": &req.username }).await? {
        let email = user_doc.get_str("email")?;
        let username = user_doc.get_str("username")?;

        let sender = format!("YWT Bot <{}>", config.smtp_username);
        let to = format!("{} <{}>", username, email);

        let content = format!("{}\n\n此邮件由 {} <{}> 触发 YWT Bot 发送。若要回复，请直接回复发件人。", req.content, user.username, admin_email);

        let email = Message::builder()
            .from(sender.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(&req.title)
            .header(ContentType::TEXT_PLAIN)
            .body(content)
            .unwrap();

        match mailer.send(&email) {
            Ok(_) => log::info!("Email sent to {}", username),
            Err(e) => {
                log::error!("Failed to send email to {}: {}", username, e);
                return Err(ApiError::new(
                    ApiErrorType::Internal,
                    format!("Failed to send email to {}: {}", username, e),
                ));
            },
        }
    }
    
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}

pub fn api_scope() -> Scope {
    web::scope("/send_email")
        .service(send_email)
        .service(send_single_email)
}
