use actix_web::web::block;
use argonautica::{Hasher, Verifier};
use chrono::{Duration, Utc};
use color_eyre::Result;
use eyre::eyre;
use futures::compat::Future01CompatExt;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::{sync::Arc};
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CryptoService {
    pub key: Arc<String>,
    pub jwt_secret: Arc<String>
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64
}

pub struct Auth {
    pub token: String
}

impl CryptoService {
    #[instrument(self, password)]
    pub async fn hash_password(&self, password: String) -> Result<String> {
        Hasher::default()
            .with_secret_key(&*self.key)
            .with_password(password)
            .hash_non_blocking()
            .compat()
            .await
            .map_err(|err| eyre!("Hashing error: {:?}", err))
    }

    pub async fn verify_password(
        &self,
        password: &str,
        password_hash: &str
    ) -> Result<bool> {
        Verifier::default()
            .with_secret_key(&*self.key)
            .with_hash(password_hash)
            .with_password(password)
            .verify_non_blocking()
            .compat()
            .await
            .map_err(|err| eyre!("Verifying password error: {}", err))
    }

    pub async fn gen_jwt(&self, user_id: Uuid) -> Result<Result<String>> {
        let jwt_key = self.jwt_secret.clone();
        Ok(block(move || {
            let headers = Header::default();
            let encoding_key = EncodingKey::from_secret(jwt_key.as_bytes());
            let now = Utc::now() + Duration::days(1); // Token Expires in 1 day
            let claims = Claims {
                sub: user_id,
                exp: now.timestamp()
            };
            encode(&headers, &claims, &encoding_key)
        })
        .await?
        .map_err(|err| eyre!("Error creating jwt token: {}", err)))
    }

    pub async fn verify_jwt(&self, token: String) -> Result<Result<TokenData<Claims>>> {
        let jwt_key = self.jwt_secret.clone();
        Ok(block(move || {
            let decoding_key = DecodingKey::from_secret(jwt_key.as_bytes());
            let validation = Validation::default();
            decode::<Claims>(&token, &decoding_key, &validation)
        })
        .await?
        .map_err(|err| eyre!("Verifying jwt token: {}", err)))

    }
}
