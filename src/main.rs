use clap::Parser;
use mongodb::Client;
use anyhow::Result;
use env_logger;
use log;
use serde_json;
use actix_web::{middleware::Logger, web, App, HttpServer, ResponseError};

use ywt::api::register;
use ywt::api::login;
use ywt::cli::Cli;
use ywt::config::Config;
use ywt::error::ApiError;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Starting YWT server...");
    let args = Cli::parse();
    let (
        bind_address,
        bind_port,
        mongo_uri,
        mongo_db,
    ) = match args.config {
        Some(path) => {
            let config_json = std::fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&config_json)?;
            (
                config.bind_address,
                config.bind_port,
                config.mongo_uri,
                config.mongo_db,
            )
        },
        None => {
            (
                "localhost".to_string(),
                8080,
                "mongodb://localhost:27017".to_string(),
                "ywt_db".to_string(),
            )
        }
    };
    let client = Client::with_uri_str(mongo_uri).await?;
    let db = client.database(&mongo_db);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .service(register::api_scope())
            .service(login::api_scope())
            .default_service(web::to(|| async {
                ApiError::new_not_found().error_response()
            }))
    })
    .bind((bind_address, bind_port))?
    .run()
    .await?;

    Ok(())
}
