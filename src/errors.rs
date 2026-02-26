// src/errors.rs

use actix_web::{ http::StatusCode, HttpResponse, ResponseError };
use deadpool_postgres::PoolError;
use derive_more::Display;
use serde::Serialize;
use tokio_pg_mapper::Error as PgMapperError;

#[derive(Display, Debug)]
pub enum ServiceError {
    #[display("잘못된 요청: {}", _0)] BadRequest(String),

    #[display("권한이 없습니다")]
    Unauthorized,

    #[display("찾을 수 없습니다")]
    NotFound,

    #[display("서버 내부 오류")] InternalServerError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl From<tokio_postgres::Error> for ServiceError {
    fn from(e: tokio_postgres::Error) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl From<PgMapperError> for ServiceError {
    fn from(e: PgMapperError) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl From<PoolError> for ServiceError {
    fn from(e: PoolError) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized => StatusCode::UNAUTHORIZED,
            ServiceError::NotFound => StatusCode::NOT_FOUND,
            ServiceError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = ErrorResponse { error: self.to_string() };
        HttpResponse::build(self.status_code()).json(body)
    }
}
