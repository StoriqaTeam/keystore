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

use config::Config;

/// Hello
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

fn get_config() -> Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
