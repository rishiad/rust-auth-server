#[macro_use]
extern crate validator_derive;

mod config;
mod models;
mod handlers;

use color_eyre::Result;
use tracing::info;
use crate::{config::Config, handlers::app_config};
use actix_web::{App, HttpServer, middleware::Logger};

#[actix_rt::main]
async fn main() -> Result<()> {
    let config: Config = Config::from_env()
                    .expect("Server Config");
    let pool = config.db_pool().await
        .expect("Database Configuration");
    info!("Starting Server...");
    HttpServer::new( move || {
        App::new()
            .wrap(Logger::default())
            .configure(app_config)
    })
        .bind(format!("{}:{}", config.host, config.port))?
        .run()
        .await?;
    Ok(())
}