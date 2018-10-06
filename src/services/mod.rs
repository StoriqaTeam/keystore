mod auth;
mod error;
mod keys;
#[cfg(test)]
mod mocks;

pub use self::auth::*;
pub use self::error::*;
pub use self::keys::*;
#[cfg(test)]
pub use self::mocks::*;

use prelude::*;

type ServiceFuture<T> = Box<Future<Item = T, Error = Error> + Send>;
