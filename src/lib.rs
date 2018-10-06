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
extern crate config as config_crate;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate hyper_tls;
extern crate r2d2;
extern crate rand;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;
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

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

use self::models::NewUser;
use self::repos::{UsersRepo, UsersRepoImpl};
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

pub fn create_user(name: &str) {
    let config = get_config();
    let database_url = config.database.url.clone();
    let manager = ConnectionManager::<PgConnection>::new(database_url.clone());
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect(&format!("Failed to connect to db with url: {}", database_url));
    let conn = db_pool
        .get()
        .expect(&format!("Failed to obtain connection from pool to db with url: {}", database_url));
    let users_repo = UsersRepoImpl::new(&conn);
    let mut new_user: NewUser = Default::default();
    new_user.name = name.to_string();
    let user = users_repo.create(new_user).expect("Failed to create user");
    println!("{}", user.authentication_token.raw())
}

fn get_config() -> Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
