use actix_web::{post, web, HttpResponse, Responder, Scope};
use rand::distr::Alphanumeric;
use serde::{Deserialize, Serialize};
use mongodb::Database;
use mongodb::bson::doc;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use rand::{rng, Rng};

use crate::error::{ApiResult, ApiError, ApiErrorType};
use crate::jwt::ClaimsValidator;
use crate::db::{
    check_user_exists,
    check_admin_exists,
    check_email_exists,
    create_user,
    AdminType, UserType
};
use crate::config::Config;
use crate::utils::{check_email, check_username, check_password, check_email_tsinghua};

#[derive(Deserialize, Serialize, Clone)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RegisterResponse {
    pub created_at: String,
}

fn check_req(req: &RegisterRequest, tsinghua: bool) -> ApiResult<()> {
    check_username(&req.username)?;
    if tsinghua {
        check_email_tsinghua(&req.email)?;
    } else {
        check_email(&req.email)?;
    }
    check_password(&req.password)?;
    Ok(())
}

#[post("")]
async fn register(
    db: web::Data<Database>,
    req: web::Json<RegisterRequest>,
    mailer: web::Data<SmtpTransport>,
    config: web::Data<Config>,
) -> ApiResult<impl Responder> {
    check_req(&req, true)?;

    if check_admin_exists(&db, &req.username).await? || check_user_exists(&db, &req.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Username already exists".to_string(),
        ));
    }
    
    if check_email_exists(&db, &req.email).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Email already exists".to_string(),
        ));
    }

    let created_at = chrono::Local::now().to_string();
    create_user::<UserType>(&db, &req.username, &req.email, &req.password, &created_at).await?;

    let collection = db.collection("stats");
    let tag_doc = doc! {
        "username": &req.username,
        "conversation": 0,
        "tags": {}, // Initialize with an empty object
    };
    collection.insert_one(tag_doc).await?;

    // generate activation code - random 32-character string
    let activation_code: String = rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    // Store the activation code in the database
    let activation_collection = db.collection("activation_codes");
    let activation_doc = doc! {
        "username": &req.username,
        "code": &activation_code,
        "created_at": &created_at,
        "expires_at": (chrono::Local::now() + chrono::Duration::days(3)).to_string(),
    };
    activation_collection.insert_one(activation_doc).await?;

    // send activation email
    let sender = format!("YWT Bot <{}>", config.smtp_username);
    let to = format!("{} <{}>", req.username, req.email);
    let email = Message::builder()
        .from(sender.parse().unwrap())
        .to(to.parse().unwrap())
        .subject("Activate your YWT account")
        .header(ContentType::TEXT_PLAIN)
        .body(format!("Hello {},\n\nYour activation code is {}\n\nThis code will expire in 3 days.\n\nBest regards,\nYWT Team", 
            req.username, activation_code))
        .unwrap();
    
    match mailer.send(&email) {
        Ok(_) => log::info!("Activation email sent to {}", req.username),
        Err(e) => log::error!("Failed to send activation email to {}: {}", req.username, e),
    }

    Ok(HttpResponse::Ok().json(RegisterResponse { created_at }))
}

#[post("/admin")]
async fn admin_register(
    db: web::Data<Database>,
    req: web::Json<RegisterRequest>,
    user: ClaimsValidator,
) -> ApiResult<impl Responder> {
    // check if user is admin
    if !check_admin_exists(&db, &user.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }
    
    check_req(&req, false)?;

    // check if user with the same username exists
    if check_user_exists(&db, &req.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }

    if check_admin_exists(&db, &req.username).await? {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Error".to_string(),
        ));
    }

    let created_at = chrono::Local::now().to_string();
    create_user::<AdminType>(&db, &req.username, &req.email, &req.password, &created_at).await?;

    Ok(HttpResponse::Ok().json(RegisterResponse { created_at }))
}

pub fn api_scope() -> Scope {
    web::scope("/register")
        .service(register)
        .service(admin_register)
}