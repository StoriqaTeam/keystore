use std::net::SocketAddr;
use std::sync::Arc;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use failure::{Compat, Fail};
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper;
use hyper::Server;
use hyper::{service::Service, Body, Request, Response};
use r2d2::Pool;

use super::config::Config;
use super::utils::{log_and_capture_error, log_error, log_warn};
use utils::read_body;

mod controllers;
mod error;
mod requests;
mod responses;
mod utils;

use r2d2;

use self::controllers::*;
use self::error::*;
use prelude::*;

#[derive(Clone)]
pub struct ApiService {
    server_address: SocketAddr,
    config: Config,
    db_pool: PgConnectionPool,
    cpu_pool: CpuPool,
}

impl ApiService {
    fn from_config(config: &Config) -> Result<Self, Error> {
        let server_address: Result<SocketAddr, Error> = format!("{}:{}", config.server.host, config.server.port)
            .parse::<SocketAddr>()
            .map_err(ectx!(
                ErrorContext::Config,
                ErrorKind::Internal =>
                config.server.host,
                config.server.port
            ));
        let server_address = server_address?;
        let database_url = config.database.url.clone();
        let manager = ConnectionManager::<PgConnection>::new(database_url.clone());
        let db_pool: Result<Pool<ConnectionManager<PgConnection>>, Error> = r2d2::Pool::builder().build(manager).map_err(ectx!(
            ErrorContext::Config,
            ErrorKind::Internal =>
            database_url
        ));
        let db_pool = db_pool?;
        let cpu_pool = CpuPool::new(config.database.thread_pool_size);
        Ok(ApiService {
            config: config.clone(),
            server_address,
            db_pool,
            cpu_pool,
        })
    }
}

impl Service for ApiService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, http_body) = req.into_parts();
        let db_pool = self.db_pool.clone();
        Box::new(
            read_body(http_body)
                .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal))
                .and_then(move |body| {
                    let router = router! {
                        GET /v1/keys => get_keys,
                        POST /v1/keys => post_keys,
                        _ => not_found,
                    };

                    let service = Arc::new(Service::new(db_pool));

                    let ctx = Context {
                        body,
                        method: parts.method.clone(),
                        uri: parts.uri.clone(),
                        headers: parts.headers,
                        keys_service: service.clone(),
                    };

                    debug!("Received request {}", ctx);

                    router(ctx, parts.method.into(), parts.uri.path())
                }).or_else(|e| match e.kind() {
                    ErrorKind::BadRequest => {
                        log_error(&e);
                        Ok(Response::builder()
                            .status(400)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Bad request"}"#))
                            .unwrap())
                    }
                    ErrorKind::Unauthorized => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(401)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Unauthorized"}"#))
                            .unwrap())
                    }
                    ErrorKind::UnprocessableEntity(errors) => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(422)
                            .header("Content-Type", "application/json")
                            .body(Body::from(format!("{}", errors)))
                            .unwrap())
                    }
                    ErrorKind::Internal => {
                        log_and_capture_error(e);
                        Ok(Response::builder()
                            .status(500)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Internal server error"}"#))
                            .unwrap())
                    }
                }),
        )
    }
}

pub fn start_server(config: Config) {
    hyper::rt::run(future::lazy(move || {
        ApiService::from_config(&config)
            .into_future()
            .and_then(move |api| {
                let api_clone = api.clone();
                let new_service = move || {
                    let res: Result<_, hyper::Error> = Ok(api_clone.clone());
                    res
                };
                let addr = api.server_address.clone();
                let server = Server::bind(&api.server_address)
                    .serve(new_service)
                    .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => addr));
                info!("Listening on http://{}", addr);
                server
            }).map_err(|e: Error| log_error(&e))
    }));
}
