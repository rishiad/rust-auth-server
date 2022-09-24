use super::AppResponse;
use crate::{
    config::crypto::{Auth, CryptoService},
    db::user::UserRepo,
    errors::AppError
};
use actix_web::{web::Data, FromRequest, HttpResponse};
use actix_web_httpauth::extractor::{basic::BasicAuth, bearer::BearerAuth};
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
                        .await
                        .map(|data| data.claims.sub)
                        .map_err(|err| {
                            debug!("Cannot verify jwt. {:?}", err);
                            AppError::NOT_AUTHORIZED
                        })
                }
            }
        }
    }
}