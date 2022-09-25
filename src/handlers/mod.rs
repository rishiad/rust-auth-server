mod user;
mod auth;

use actix_web::{web, web::ServiceConfig, HttpResponse, error};
use user::{create_user, me};
use auth::auth;

use crate::errors::AppError;

type AppResult<T> = Result<T, AppError>;
type AppResponse = AppResult<HttpResponse>;

pub fn app_config(config: &mut ServiceConfig) {
    let json_config = web::JsonConfig::default()
    .limit(4096)
    .error_handler(|err, _req| {
        // create custom error response
        error::InternalError::from_response(err, HttpResponse::Conflict().finish())
            .into()
    });

    
    let health_resource = web::resource("/")
        .route(web::get().to(health));
    let register = web::resource("/register").app_data(json_config).route(web::post().to(create_user));
    
    let auth = web::resource("/auth").route(web::post().to(auth));

    let me = web::resource("/me").route(web::get().to(me));
    
    config
        .service(health_resource)
        .service(register)
        .service(auth)
        .service(me);
}

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}