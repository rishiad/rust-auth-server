use actix_web::{HttpResponse, web::{Json, Data}};
use sqlx::postgres::PgDatabaseError;
use tracing::{debug};
use color_eyre::Result;
use validator::Validate;

use crate::{models::user::{NewUser, User}, db::{user::UserRepo, self}, config::crypto::CryptoService, errors::{self, AppError}};

use super::{AppResponse, auth::AuthedUser};


pub async fn create_user(
    user: Json<NewUser>,
    repo: UserRepo,
    crypto_service: Data<CryptoService>
) -> AppResponse {
    match user.validate() {
        Ok(_) => Ok(()),
        Err(errors) => {
            let error_map = errors.field_errors();
            debug!("ye");
            let message = if error_map.contains_key("username") {
                format!("Invalid username, \"{}\" is too short", user.username)
            } else if error_map.contains_key("email") {
                format!("Invalid email address \"{}\"", user.email)
            } else if error_map.contains_key("password") {
                "Invalid password. Too short".to_string()
            } else {
                "Invalid input.".to_string()
            };

            Err(AppError::INVALID_INPUT.message(message))
        }
    }?;

    let result: Result<User> = repo.create(user.0, crypto_service.as_ref()).await;

    match result {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(error) => {
            let pg_error: &PgDatabaseError = 
                error
                    .root_cause()
                    .downcast_ref::<PgDatabaseError>()
                    .ok_or_else(|| {
                        debug!("Error creating user. {:?}", error);
                        AppError::INTERNAL_ERROR
                    })?;
 
                let error =  match (pg_error.code(), pg_error.column()) {
                    (db::UNIQUE_VIOLATION_CODE, std::option::Option::Some("email")) => {
                        AppError::INVALID_INPUT.message("Email already exists.".to_string())
                    },
                    (db::UNIQUE_VIOLATION_CODE, std::option::Option::Some("username")) => {
                        AppError::INVALID_INPUT.message("Username already exists.".to_string())
                    },
                    (db::UNIQUE_VIOLATION_CODE, None) => {
                        AppError::INVALID_INPUT.message("Username or email already exists.".to_string())
                    },    
                    _ => {
                        debug!("Error creating user. {:?}", pg_error);
                        AppError::INTERNAL_ERROR.into()
                    }          
                };
                Err(error)
        }
    }
}

pub async fn me(
    user: AuthedUser,
    repo: UserRepo,
) -> AppResponse {
    let user = repo
        .find_by_id(user.0)
        .await?
        .ok_or(AppError::INTERNAL_ERROR)?;

    Ok(HttpResponse::Ok().json(user))
}