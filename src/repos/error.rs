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
    #[fail(display = "database error - internal")]
    Internal,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "database source - error inside of Diesel library")]
    Diesel,
    #[fail(display = "database source - error inside of r2d2 library")]
    R2D2,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "database context - error getting connection")]
    Connection,
}

derive_error_impls!();
