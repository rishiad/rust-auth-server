mod user;
mod auth;

use actix_web::{web, web::ServiceConfig, HttpResponse};
use user::create_user;
pub fn app_config(config: &mut ServiceConfig) {
    let health_resource = web::resource("/")
        .route(web::get().to(health));
    let register = web::resource("/register").route(web::post().to(create_user));
        config.service(health_resource);
}

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}