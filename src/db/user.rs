
use std::ops::Deref;
use std::sync::Arc;
use actix_web::FromRequest;
use actix_web::web::Data;
use futures::future::{Ready, ready};
use sqlx::{PgPool};
use  sqlx::query::QueryAs;
use tracing::instrument;
use crate::errors::AppError;
use crate::{models::user::{NewUser, User}, config::crypto::CryptoService};
use color_eyre::Result;


pub struct UserRepo {
    pool: Arc<PgPool>
}

impl UserRepo {
    pub fn new(pool: Arc<PgPool>) -> Self {
       Self { pool } 
    }
    
    pub async fn create(&self, 
        new_user: NewUser, 
        crypto_service: &CryptoService) -> Result<User> {
            let password_hash = crypto_service.hash_password(new_user.password)
                .await?;

            let user = sqlx::query_as::<_, User>(
                "insert into users (username, email, password_hash) values ($1, $2, $3) returning *"
            )
            .bind(new_user.username)
            .bind(new_user.email)
            .bind(password_hash)
            .fetch_one(&*self.pool)
            .await?;
        Ok(user)
    }
}

impl FromRequest for UserRepo {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;
    // type Config = ();

    #[instrument(skip(req, payload))]
    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let pool_result = Data::<PgPool>::from_request(req, payload).into_inner();

        match pool_result {
            Ok(pool) => ready(Ok(UserRepo::new(pool.deref().clone()))),
            _=> ready(Err(AppError::NOT_AUTHORIZED.default())),
        }
    }

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        Self::from_request(req, &mut actix_web::dev::Payload::None)
    }
}