use std::sync::Arc;

use super::error::*;
use blockchain::BlockchainService;
use models::*;
use prelude::*;
use repos::{DbExecutor, Isolation, KeysRepo};

pub trait MetricsService: Send + Sync + 'static {
    fn get_metrics(&self) -> Box<Future<Item = Metrics, Error = Error> + Send>;
}

#[derive(Clone)]
pub struct MetricsServiceImpl<E: DbExecutor> {
    keys_repo: Arc<KeysRepo>,
    blockchain_service: Arc<BlockchainService>,
    db_executor: E,
}

impl<E: DbExecutor> MetricsServiceImpl<E> {
    pub fn new(keys_repo: Arc<KeysRepo>, blockchain_service: Arc<BlockchainService>, db_executor: E) -> Self {
        MetricsServiceImpl {
            keys_repo,
            db_executor,
            blockchain_service,
        }
    }
}

impl<E: DbExecutor> MetricsService for MetricsServiceImpl<E> {
    fn get_metrics(&self) -> Box<Future<Item = Metrics, Error = Error> + Send> {
        let self_ = self.clone();
        self.db_executor
            .execute_transaction_with_isolation(Isolation::RepeatableRead, move || {
                let keys = self_.keys_repo.all().map_err(ectx!(try ErrorKind::Internal))?;
                let mut total_keys: u64 = 0;
                let mut failed_derivations_count: u64 = 0;
                for key in keys {
                    total_keys += 1;

                    let Key {
                        currency,
                        private_key,
                        blockchain_address,
                        ..
                    } = key;

                    let derived = self_
                        .blockchain_service
                        .derive_address(currency.clone(), private_key.clone())
                        .map_err(ectx!(try ErrorKind::Internal => currency, private_key))?;

                    if blockchain_address != derived {
                        failed_derivations_count += 1;
                    }
                }
                Ok(Metrics {
                    total_keys,
                    failed_derivations_count,
                })
            })
    }
}
