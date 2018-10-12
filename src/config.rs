use serde;
use serde::{Deserialize, Deserializer};
use std::env;

use sentry_integration::SentryConfig;

use config_crate::{Config as RawConfig, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub blockchain: Blockchain,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Blockchain {
    pub stq_gas_limit: usize,
    pub stq_contract_address: String,
    pub stq_transfer_method_number: String,
    pub ethereum_chain_id: Option<u64>,
    #[serde(deserialize_with = "deserialize_btc_network")]
    pub btc_network: BtcNetwork,
}

#[derive(Debug, Deserialize, Clone)]
pub enum BtcNetwork {
    Main,
    Test,
}

fn deserialize_btc_network<'de, D>(de: D) -> Result<BtcNetwork, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(de)?;
    match s.as_ref() {
        "test" => Ok(BtcNetwork::Test),
        "main" => Ok(BtcNetwork::Main),
        other => Err(serde::de::Error::custom(format!("unknown network: {}", other))),
    }
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
