use std::time::Duration;
use eyre::WrapErr;
use color_eyre::Result;
use serde::Deserialize;
use dotenv::dotenv;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing_subscriber::EnvFilter;
use tracing::{info, instrument};


#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: i32,
    pub database_url: String
}

impl Config {

    #[instrument]
    pub fn from_env() -> Result<Config> {
        dotenv().ok();
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        info!("Loading config...");
        let mut c = config::Config::new();

        c.merge(config::Environment::default())?;
        c.try_into()
            .context("loading config from env")
    }

    pub async fn db_pool(&self) -> Result<PgPool> {
        info!("Creating DB connection pool...");

        Ok(PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.database_url)
            .await
            .context("Connecting to database pool...")?)
    }
}