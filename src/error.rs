use std::fmt::format;

use actix_web::error::BlockingError;
use actix_web::HttpResponse;
use derive_more::{Display, Error};
use failure::Fail;
use image::ImageError;
use log::{error, warn};
use paperclip::actix::api_v2_errors;
use serde_json::{Map as JsonMap, Value as JsonValue};

#[derive(Display, Debug, Error)]
pub struct FatalError {
    pub message: String,
}

#[api_v2_errors(code = 400, code = 404, code = 401, code = 500)]
#[derive(Debug, Fail)]
pub enum AppError {
    #[fail(display = "Argon2 Error {}", _0)]
    Argon2Error(argon2::Error),

    #[fail(display = "sqlx error Error ")]
    SqlxError(sqlx::Error),

    #[fail(display = "Blocking Error")]
    BlockingError(BlockingError),

    #[fail(display = "Blocking Error")]
    ImageError(ImageError),

    #[fail(display = "Jwt Encode Error: {}", _0)]
    JwtError(jsonwebtoken::errors::Error),

    #[fail(display = "Validation Error {}", _0)]
    ValidationErrors(validator::ValidationErrors),

    #[fail(display = "Fatal Error {}", _0)]
    FatalErrorWithMessage(FatalError),

    #[fail(display = "Fatal Error")]
    _FatalError,

    #[fail(display = "Unauthorized")]
    HTTPUnauthorized,

    #[fail(display = "Unauthorized")]
    HTTPForbidden,

    #[fail(display = "NotFound")]
    _HTTPNotFound,

    #[fail(display = "Bad request")]
    HTTPBadRequest(String),

    #[fail(display = "Bad request")]
    WrongRowsChanged(u64),
}

impl From<BlockingError> for AppError {
    fn from(e: BlockingError) -> Self {
        AppError::BlockingError(e)
    }
}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Argon2Error(ref err) => internal_server_error_response(err),
            AppError::SqlxError(ref err) => match err {
                sqlx::Error::RowNotFound => not_found_response(),
                _ => internal_server_error_response(&self),
            },
            AppError::JwtError(ref err) => unauthorized_response(&format!("{:?}", err)),
            AppError::HTTPUnauthorized => unauthorized_response("Unauthorized"),
            AppError::_HTTPNotFound => not_found_response(),
            AppError::HTTPForbidden => HttpResponse::Forbidden().finish(),
            AppError::_FatalError => fatal_error(),
            AppError::BlockingError(ref err) => internal_server_error_response(err),
            AppError::FatalErrorWithMessage(ref err) => internal_server_error_response(err),
            AppError::HTTPBadRequest(ref message) => bad_request_error(message),
            AppError::ValidationErrors(ref errs) => unprocessable_entity_response(&validation_errs_to_json(errs)),
            AppError::WrongRowsChanged(message) => bad_request_error(&format!("Wrong rows affected {message}")),
            AppError::ImageError(e) => bad_request_error(&format!("{e:?}")),
        }
    }
}

impl From<argon2::Error> for AppError {
    fn from(error: argon2::Error) -> Self {
        AppError::Argon2Error(error)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AppError::JwtError(error)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(errors: sqlx::Error) -> Self {
        AppError::SqlxError(errors)
    }
}
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        AppError::ValidationErrors(errors)
    }
}

impl From<FatalError> for AppError {
    fn from(errors: FatalError) -> Self {
        AppError::FatalErrorWithMessage(errors)
    }
}

impl From<ImageError> for AppError {
    fn from(errors: ImageError) -> Self {
        AppError::ImageError(errors)
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(errors: chrono::ParseError) -> Self {
        warn!("{}", errors);
        AppError::HTTPBadRequest("".parse().unwrap())
    }
}

fn validation_errs_to_json(errors: &validator::ValidationErrors) -> JsonValue {
    let mut err_map = JsonMap::new();
    for (field, errors) in errors.clone().field_errors().iter() {
        let first_error = errors.first();
        let mut error_message = "";
        if let Some(error) = first_error {
            match &error.message {
                Some(message) => error_message = message,
                None => {
                    if error.code == "length" {
                        error_message = "Required"
                    }
                }
            }
        }

        err_map.insert(field.to_string(), json!(error_message));
    }
    json!({ "err_objects": err_map })
}

fn unprocessable_entity_response(json: &serde_json::Value) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY).json(json)
}

fn internal_server_error_response<T>(err: T) -> HttpResponse
where
    T: std::fmt::Debug,
{
    error!("internal_server_error: {:?}", err);
    HttpResponse::InternalServerError().json(json!({"error": "Internal Server Error"}))
}

fn not_found_response() -> HttpResponse {
    HttpResponse::NotFound().json(json!({"error": "not found"}))
}

fn unauthorized_response(message: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({ "error": message }))
}

fn bad_request_error(message: &str) -> HttpResponse {
    if message.is_empty() {
        HttpResponse::BadRequest().finish()
    } else {
        HttpResponse::BadRequest().json(json!({ "error": message }))
    }
}

fn fatal_error() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}
