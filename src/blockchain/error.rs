use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    #[fail(display = "blockchain error - internal")]
    Internal { error: InternalError, source: Option<ErrorSource> },
    #[fail(display = "blockchain error - validation")]
    Validation(ValidationError),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail, PartialEq, Eq, Serialize, Deserialize)]
pub enum InternalError {
    #[fail(display = "malformed method number")]
    MalformedMethodNumber { value: String },
    #[fail(display = "malformed STQ contract address")]
    MalformedStqContractAddress { value: String },
    #[fail(display = "overflow")]
    Overflow,
    #[fail(display = "error generating random number")]
    Random,
    #[fail(display = "serialization error")]
    Serialization,
    #[fail(display = "error signing message")]
    Signature,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationError {
    #[fail(display = "malformed address")]
    MalformedAddress { value: String },
    #[fail(display = "malformed hex string")]
    MalformedHexString { value: String },
    #[fail(display = "malformed private key")]
    MalformedPrivateKey { value: String },
    #[fail(display = "missing nonce")]
    MissingNonce,
    #[fail(display = "not enough sathoshis in utxos")]
    NotEnoughUtxo,
    #[fail(display = "overflow")]
    Overflow { number: String },
    #[fail(display = "unsupported blockchain address type")]
    UnsupportedAddressType { value: String },
    #[fail(display = "unsupported currency")]
    UnsupportedCurrency { value: String },
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail, Serialize, Deserialize)]
pub enum ErrorSource {
    #[fail(display = "OS rng")]
    Random,
    #[fail(display = "serde")]
    Serde,
    #[fail(display = "transaction signer")]
    Signer,
}

derive_error_impls!();
