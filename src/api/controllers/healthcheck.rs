use super::Context;
use super::ControllerFuture;
use futures::prelude::*;
use hyper::{Body, Response};

pub fn get_healthcheck(_ctx: &Context) -> ControllerFuture {
    Box::new(
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(r#""Ok""#))
            .unwrap())
        .into_future(),
    )
}
