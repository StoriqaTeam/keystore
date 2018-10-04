use std::env;

use sentry_integration::SentryConfig;

use config_crate::{Config as RawConfig, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub sentry: Option<SentryConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub url: String,
    pub thread_pool_size: usize,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Merge development.toml if RUN_MODE variable is not set
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        s.merge(Environment::with_prefix("STQ_PAYMENTS"))?;
        s.try_into()
    }
}
