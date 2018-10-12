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
use blockchain::BlockchainServiceImpl;
use models::*;
use prelude::*;
use repos::{DbExecutorImpl, KeysRepoImpl, UsersRepoImpl};
use serde_json;
use services::{AuthServiceImpl, KeysServiceImpl, TransactionsServiceImpl};

#[derive(Clone)]
pub struct ApiService {
    server_address: SocketAddr,
    config: Config,
    db_pool: PgPool,
    cpu_pool: CpuPool,
}

impl ApiService {
    fn from_config(config: &Config) -> Result<Self, Error> {
        let server_address = format!("{}:{}", config.server.host, config.server.port)
            .parse::<SocketAddr>()
            .map_err(ectx!(
                try
                ErrorContext::Config,
                ErrorKind::Internal =>
                config.server.host,
                config.server.port
            ))?;
        let database_url = config.database.url.clone();
        let manager = ConnectionManager::<PgConnection>::new(database_url.clone());
        let db_pool = r2d2::Pool::builder().build(manager).map_err(ectx!(try
            ErrorContext::Config,
            ErrorKind::Internal =>
            database_url
        ))?;
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
        let thread_pool = self.cpu_pool.clone();
        let db_executor = DbExecutorImpl::new(db_pool.clone(), thread_pool.clone());
        let config = self.config.clone();
        Box::new(
            read_body(http_body)
                .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal))
                .and_then(move |body| {
                    let router = router! {
                        GET /v1/users/{user_id: UserId}/keys => get_keys,
                        POST /v1/users/{user_id: UserId}/keys => post_keys,
                        POST /v1/transactions => post_transactions,
                        _ => not_found,
                    };

                    let auth_service = Arc::new(AuthServiceImpl::new(Arc::new(UsersRepoImpl), db_executor.clone()));
                    let blockchain_service = Arc::new(BlockchainServiceImpl::new(
                        config.blockchain.stq_gas_limit.clone(),
                        config.blockchain.stq_contract_address.clone(),
                        config.blockchain.stq_transfer_method_number.clone(),
                        config.blockchain.ethereum_chain_id.clone(),
                        config.blockchain.btc_network.clone(),
                    ));
                    let keys_repo = Arc::new(KeysRepoImpl);
                    let keys_service = Arc::new(KeysServiceImpl::new(
                        auth_service.clone(),
                        blockchain_service.clone(),
                        keys_repo.clone(),
                        db_executor.clone(),
                    ));

                    let transactions_service = Arc::new(TransactionsServiceImpl::new(
                        auth_service.clone(),
                        keys_repo.clone(),
                        blockchain_service.clone(),
                        db_executor.clone(),
                    ));

                    let ctx = Context {
                        body,
                        method: parts.method.clone(),
                        uri: parts.uri.clone(),
                        headers: parts.headers,
                        keys_service,
                        transactions_service,
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
                    ErrorKind::NotFound => {
                        log_error(&e);
                        Ok(Response::builder()
                            .status(404)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Not found"}"#))
                            .unwrap())
                    }

                    ErrorKind::UnprocessableEntity(errors) => {
                        log_warn(&e);
                        let errors =
                            serde_json::to_string(&errors).unwrap_or(r#"{"message": "unable to serialize validation errors"}"#.to_string());
                        Ok(Response::builder()
                            .status(422)
                            .header("Content-Type", "application/json")
                            .body(Body::from(errors))
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
