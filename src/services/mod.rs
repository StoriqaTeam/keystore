mod auth;
mod error;
mod keys;

pub use self::auth::*;
pub use self::error::*;
pub use self::keys::*;

use prelude::*;

type ServiceFuture<T> = Box<Future<Item = T, Error = Error> + Send>;
