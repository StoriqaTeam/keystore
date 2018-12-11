use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};
use validator::ValidationErrors;

use blockchain::{ErrorKind as BlockchainErrorKind, ValidationError as BlockchainValidationError};
use repos::{Error as ReposError, ErrorKind as ReposErrorKind};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "service error - unauthorized")]
    Unauthorized,
    #[fail(display = "service error - malformed input")]
    MalformedInput,
    #[fail(display = "service error - not found")]
    NotFound,
    #[fail(display = "service error - validation - {}", _0)]
    Validation(ValidationError),
    #[fail(display = "service error - internal error")]
    Internal,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "service error source - r2d2")]
    R2D2,
}

#[derive(Clone, Debug, Fail, PartialEq, Serialize)]
pub enum ValidationError {
    #[fail(display = "blockchain - {}", _0)]
    Blockchain(BlockchainValidationError),
    #[fail(display = "validator - {}", _0)]
    Validator(ValidationErrors),
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "service error context - no auth token received")]
    NoAuthToken,
    #[fail(display = "service error context - tried to access resources that doesn't belong to user")]
    NotOwnResources,
    #[fail(display = "service error context - no wallet with this address found")]
    NoWallet,
    #[fail(display = "service error context - no system user found")]
    NoSystemUser,
    #[fail(display = "service error context - signing transaction")]
    SigningTransaction,
    #[fail(display = "service error context - currency is not supported")]
    NotSupportedCurrency,
}

derive_error_impls!();

impl From<ReposError> for Error {
    fn from(e: ReposError) -> Error {
        let kind: ErrorKind = e.kind().into();
        e.context(kind).into()
    }
}

impl From<ReposErrorKind> for ErrorKind {
    fn from(e: ReposErrorKind) -> ErrorKind {
        match e {
            ReposErrorKind::Internal => ErrorKind::Internal,
            ReposErrorKind::Constraints(validation_errors) => ErrorKind::Validation(ValidationError::Validator(validation_errors)),
        }
    }
}

impl From<BlockchainErrorKind> for ErrorKind {
    fn from(e: BlockchainErrorKind) -> ErrorKind {
        match e {
            BlockchainErrorKind::Internal { .. } => ErrorKind::Internal,
            BlockchainErrorKind::Validation(error) => ErrorKind::Validation(ValidationError::Blockchain(error)),
        }
    }
}
