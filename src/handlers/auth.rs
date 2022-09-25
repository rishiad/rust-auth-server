use super::AppResponse;
use crate::{
    config::crypto::{Auth, CryptoService},
    db::user::UserRepo,
    errors::AppError
};
use actix_web::{web::Data, FromRequest, HttpResponse};
use actix_web_httpauth::{extractors::{basic::BasicAuth, bearer::BearerAuth}};
use futures::future::{ready, BoxFuture, self};
use tracing::{debug, instrument};
use uuid::Uuid;

pub struct AuthedUser(pub Uuid);

impl FromRequest for AuthedUser {
    type Error = AppError;
    type Future = BoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let bearer_result = BearerAuth::from_request(req, payload).into_inner();
        let repo_result = UserRepo::from_request(req, payload).into_inner();
        let crypto_service_result = Data::<CryptoService>::from_request(req, payload).into_inner();

        match (bearer_result, repo_result, crypto_service_result) {
            (Ok(bearer), Ok(repo), Ok(crypto_service)) => {
                let future = async move {
                    let user_id: Uuid = crypto_service
                        .verify_jwt(bearer.token().to_string())
                        .await?
                        .map(|data| data.claims.sub)
                        .map_err(|err| {
                            debug!("Cannot verify jwt. {:?}", err);
                            AppError::NOT_AUTHORIZED
                        })?;
                        Ok(AuthedUser(user_id))
                };
                Box::pin(future)
            }
            _ => {
                let error = ready(Err(AppError::NOT_AUTHORIZED.into()));
                Box::pin(error)
            }
        }
    }

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        Self::from_request(req, &mut actix_web::dev::Payload::None)
    }
}

pub async fn auth (
    basic : BasicAuth,
    repo: UserRepo,
    hashing: Data<CryptoService>
) -> AppResponse {
    let username = basic.user_id();
    let password = basic
        .password()
        .ok_or_else(|| {
            debug!("Invaild request. Missing Basic Auth.");
            AppError::INVALID_CREDENTIALS
        })?;

    let user = repo
        .find_by_username(username)
        .await?
        .ok_or_else(|| {
            debug!("User doesn't exsit.");
            AppError::INVALID_CREDENTIALS
        })?;

    let valid = hashing
        .verify_password(password, &user.pass_hash)
        .await?;

        if valid {
            let token = hashing.gen_jwt(user.id).await?;
            Ok(HttpResponse::Ok().json(Auth { token : format!("{:?}", token) }))
        } else {
            debug!("Invaild password.");
            Err(AppError::INVALID_CREDENTIALS.into())
        }
}