#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate http_router;
#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate sentry;

extern crate base64;
extern crate bitcrypto as btccrypto;
extern crate chain as btcchain;
extern crate config as config_crate;
extern crate ethcore_transaction;
extern crate ethereum_types;
extern crate ethkey;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate hyper_tls;
extern crate keys as btckey;
extern crate primitives as btcprimitives;
extern crate r2d2;
extern crate rand;
extern crate regex;
extern crate rlp;
extern crate script as btcscript;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;
extern crate serialization as btcserialization;
#[cfg(test)]
extern crate tokio_core;
extern crate uuid;
extern crate validator;

#[macro_use]
mod macros;
mod api;
mod blockchain;
mod config;
mod models;
mod prelude;
mod repos;
mod schema;
mod sentry_integration;
mod services;
mod utils;

use std::str::FromStr;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use futures_cpupool::CpuPool;
use uuid::Uuid;

use self::models::*;
use self::prelude::*;
use self::repos::{DbExecutor, DbExecutorImpl, Error as ReposError, UsersRepo, UsersRepoImpl};
use config::Config;

pub fn hello() {
    println!("Hello world");
}

pub fn print_config() {
    println!("Parsed config: {:?}", get_config());
}

pub fn start_server() {
    let config = get_config();
    // Prepare sentry integration
    let _sentry = sentry_integration::init(config.sentry.as_ref());
    api::start_server(config);
}

pub fn create_user(name: &str, uuid: &str, token: &str) {
    let id = UserId::new(Uuid::from_str(uuid).expect(&format!("Cannot parse uuid `{}`", uuid)));
    let token = AuthenticationToken::new(token.to_string());
    let config = get_config();
    let db_pool = create_db_pool(&config);
    let cpu_pool = CpuPool::new(1);
    let users_repo = UsersRepoImpl;
    let db_executor = DbExecutorImpl::new(db_pool, cpu_pool);
    let new_user: NewUser = NewUser {
        id,
        name: name.to_string(),
        authentication_token: token,
    };
    let fut = db_executor.execute(move || -> Result<(), ReposError> {
        let user = users_repo.create(new_user).expect("Failed to create user");
        println!("{}", user.authentication_token.raw());
        Ok(())
    });
    hyper::rt::run(fut.map(|_| ()).map_err(|_| ()));
}

fn create_db_pool(config: &Config) -> PgPool {
    let database_url = config.database.url.clone();
    let manager = ConnectionManager::<PgConnection>::new(database_url.clone());
    r2d2::Pool::builder()
        .build(manager)
        .expect(&format!("Failed to connect to db with url: {}", database_url))
}

fn get_config() -> Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
