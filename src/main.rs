use core::panic;

use clap::Parser;
use dotenvy::dotenv;
use mongodb::Client;
use mongodb::bson::doc;
use anyhow::Result;
use env_logger;
use log;
use serde_json;
use actix_web::{middleware::Logger, web, App, HttpServer, ResponseError};
use actix_cors::Cors;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use lettre::transport::smtp::authentication::Credentials;
use lettre::SmtpTransport;

use ywt::api::{register, login, profile, modify, stats, problem, send_email, verify_email, users};
use ywt::cli::Cli;
use ywt::config::Config;
use ywt::error::ApiError;
use ywt::api::problem::QBankEntry;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Starting YWT server...");
    let args = Cli::parse();
    let (
        config,
        bind_address,
        bind_port,
        mongo_uri,
        mongo_db,
        admin_username,
        admin_email,
        smtp_server,
        smtp_port,
        smtp_username,
    ) = match args.config {
        Some(path) => {
            let config_json = std::fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&config_json)?;
            (
                config.clone(),
                config.bind_address,
                config.bind_port,
                config.mongo_uri,
                config.mongo_db,
                config.admin_username,
                config.admin_email,
                config.smtp_server,
                config.smtp_port,
                config.smtp_username,
            )
        },
        None => {
            panic!("No config file provided. Please provide a config file using --config <path>");
        }
    };

    let qbank_path = "./Q_bank/Q_bank.json";
    let qbank_json_string = std::fs::read_to_string(qbank_path)
        .expect(&format!("Failed to read Q_bank file at {}", qbank_path));
    let qbank_data: Vec<QBankEntry> = serde_json::from_str(&qbank_json_string)
        .expect("Failed to parse Q_bank.json");
    log::info!("Successfully loaded {} entries from {}", qbank_data.len(), qbank_path);

    let smtp_password = std::env::var("YWT_SMTP_PASSWORD").unwrap_or_else(|_| "your_password".to_string());
    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = SmtpTransport::starttls_relay(&smtp_server)
        .unwrap()
        .port(smtp_port)
        .credentials(creds)
        .build();

    let client = Client::with_uri_str(mongo_uri).await?;
    let db = client.database(&mongo_db);

    let admin_password = std::env::var("YWT_ADMIN_PASSWORD").unwrap_or_else(|_| "adminpassword".to_string());
    // check if the admins collection is empty
    let collection = db.collection::<mongodb::bson::Document>("admins");
    let admin_count = collection.count_documents(doc! {}).await?;

    if admin_count == 0 {
        // create the admin user
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(admin_password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        collection.insert_one(doc! {
            "username": &admin_username,
            "password": password_hash,
            "email": &admin_email,
            "created_at": chrono::Utc::now().to_rfc3339(), // Use UTC time and standard format
        }).await?;
        log::info!("Admin user created: {}", admin_username);
    } else {
        log::info!("Admin collection is not empty, skipping admin creation.");
    }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(mailer.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(qbank_data.clone()))
            .service(register::api_scope())
            .service(login::api_scope())
            .service(profile::api_scope())
            .service(modify::api_scope())
            .service(stats::api_scope())
            .service(problem::api_scope())
            .service(send_email::api_scope())
            .service(verify_email::api_scope())
            .service(users::api_scope())
            .default_service(web::to(|| async {
                ApiError::new_not_found().error_response()
            }))
    })
    .bind((bind_address, bind_port))?
    .run()
    .await?;

    Ok(())
}
