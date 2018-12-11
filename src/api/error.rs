use failure::{Backtrace, Context, Fail};
use services::ErrorKind as ServiceErrorKind;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "controller error - unauthorized")]
    Unauthorized,
    #[fail(display = "controller error - bad request")]
    BadRequest,
    #[fail(display = "controller error - not found")]
    NotFound,
    #[fail(display = "controller error - unprocessable entity")]
    UnprocessableEntity(serde_json::Value),
    #[fail(display = "controller error - internal error")]
    Internal,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "controller source - error inside of Hyper library")]
    Hyper,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "controller context - error parsing config data")]
    Config,
    #[fail(display = "controller context - error converting json data from request")]
    RequestJson,
    #[fail(display = "controller context - error parsing bytes into utf8 from request")]
    RequestUTF8,
    #[fail(display = "controller context - missing query despite required params")]
    RequestMissingQuery,
    #[fail(display = "controller context - failed to extract query params")]
    RequestQueryParams,
    #[fail(display = "controller context - error converting json data from request")]
    ResponseJson,
}

derive_error_impls!();

impl From<ServiceErrorKind> for ErrorKind {
    fn from(err: ServiceErrorKind) -> Self {
        match err {
            ServiceErrorKind::Internal { .. } => ErrorKind::Internal,
            ServiceErrorKind::Unauthorized => ErrorKind::Unauthorized,
            ServiceErrorKind::MalformedInput => ErrorKind::BadRequest,
            ServiceErrorKind::Validation(error) => match serde_json::to_value(error) {
                Ok(json) => ErrorKind::UnprocessableEntity(json),
                Err(_) => ErrorKind::Internal,
            },
            ServiceErrorKind::NotFound => ErrorKind::NotFound,
        }
    }
}
