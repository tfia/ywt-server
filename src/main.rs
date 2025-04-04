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

use ywt::api::{register, login, profile, stats};
use ywt::cli::Cli;
use ywt::config::Config;
use ywt::error::ApiError;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Starting YWT server...");
    let args = Cli::parse();
    let (
        bind_address,
        bind_port,
        mongo_uri,
        mongo_db,
        admin_username,
        admin_email,
    ) = match args.config {
        Some(path) => {
            let config_json = std::fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&config_json)?;
            (
                config.bind_address,
                config.bind_port,
                config.mongo_uri,
                config.mongo_db,
                config.admin_username,
                config.admin_email
            )
        },
        None => {
            (
                "localhost".to_string(),
                8080,
                "mongodb://localhost:27017".to_string(),
                "ywt_db".to_string(),
                "admin".to_string(),
                "test@example.com".to_string(),
            )
        }
    };
    let client = Client::with_uri_str(mongo_uri).await?;
    let db = client.database(&mongo_db);

    let admin_password = std::env::var("YWT_ADMIN_PASSWORD").unwrap_or_else(|_| "adminpassword".to_string());
    // check if the admin user exists
    let collection = db.collection("admins");
    let existing_admin = collection
        .find_one(doc! { "username": &admin_username })
        .await?;
    if existing_admin.is_none() {
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
            "created_at": chrono::Local::now().to_string(),
        }).await?;
        log::info!("Admin user created: {}", admin_username);
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
            .service(register::api_scope())
            .service(login::api_scope())
            .service(profile::api_scope())
            .service(stats::api_scope())
            .default_service(web::to(|| async {
                ApiError::new_not_found().error_response()
            }))
    })
    .bind((bind_address, bind_port))?
    .run()
    .await?;

    Ok(())
}
