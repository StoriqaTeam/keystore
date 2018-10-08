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
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "blockchain source - error generating random numer using OS rng")]
    Random,
}

derive_error_impls!();
