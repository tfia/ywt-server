use actix_web::{get, web, HttpResponse, Responder, Scope};
use lettre::message::header::ContentType;
use mongodb::{Database, Collection};
use mongodb::bson::{doc, Document};
use futures::TryStreamExt;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::jwt::ClaimsValidator;
use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::db;

#[get("")]
async fn send_stats_email(
    db: web::Data<Database>,
    smtp: web::Data<(String, String, u16)>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    // Verify if the user is an admin
    if !db::check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Admin access required".to_string(),
        ));
    }
    
    // Configure SMTP Transport
    let (smtp_server, smtp_username, smtp_port) = smtp.as_ref().clone();
    let smtp_password = std::env::var("YWT_SMTP_PASSWORD").unwrap_or_else(|_| "your_password".to_string());
    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
    let mailer = SmtpTransport::starttls_relay(&smtp_server.to_string())
        .unwrap()
        .port(smtp_port)
        .credentials(creds)
        .build();

    // Get all users and their stats
    let users_collection: Collection<Document> = db.collection("users");
    let mut users_cursor = users_collection.find(doc! {}).await?;

    let stats_collection: Collection<Document> = db.collection("stats");
    while let Some(user_doc) = users_cursor.try_next().await? {
        let email = user_doc.get_str("email")?;
        let username = user_doc.get_str("username")?;

        let sender = format!("YWT Bot <{}>", smtp_username.to_string());
        let to = format!("{} <{}>", username, email);

        if let Some(stats_doc) = stats_collection.find_one(doc! { "username": username }).await? {
            let email = Message::builder()
                .from(sender.parse().unwrap())
                .to(to.parse().unwrap())
                .subject("Your YWT Stats")
                .header(ContentType::TEXT_PLAIN)
                .body(format!("Here are your stats:\n{:?}", stats_doc))
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

pub fn api_scope() -> Scope {
    web::scope("/send_email")
        .service(send_stats_email)
}
