use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Fail)]
pub enum ErrorKind {
    #[fail(display = "blockchain error - internal")]
    Internal,
    #[fail(display = "blockchain error - malformed address")]
    MalformedAddress,
    #[fail(display = "blockchain error - missing nonce")]
    MissingNonce,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Fail)]
pub enum ErrorContext {
    #[fail(display = "blockchain context - error converting to H160")]
    H160Convert,
    #[fail(display = "blockchain context - error serializing blockchain address")]
    AddressConvert,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "blockchain source - error generating random numer using OS rng")]
    Random,
    #[fail(display = "blockchain source - error in transaction signing")]
    Signer,
}

derive_error_impls!();
