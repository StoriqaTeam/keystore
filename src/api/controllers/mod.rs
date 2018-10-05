use std::fmt::{self, Display};
use std::sync::Arc;

use futures::prelude::*;
use hyper::{header::HeaderValue, Body, HeaderMap, Method, Response, Uri};

use super::error::*;
use prelude::*;
use services::KeysService;

mod fallback;
mod keys;

pub use self::fallback::*;
pub use self::keys::*;

pub type ControllerFuture = Box<Future<Item = Response<Body>, Error = Error> + Send>;

#[derive(Clone)]
pub struct Context {
    pub body: Vec<u8>,
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap<HeaderValue>,
    pub keys_service: Arc<KeysService>,
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!(
            "{} {}, headers: {:#?}, body: {:?}",
            self.method,
            self.uri,
            self.headers,
            String::from_utf8(self.body.clone()).ok()
        ))
    }
}
