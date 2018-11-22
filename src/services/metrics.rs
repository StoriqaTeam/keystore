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
                for key in keys {
                    assert_eq!(
                        key.blockchain_address,
                        self_
                            .blockchain_service
                            .derive_address(key.currency, key.private_key)
                            .map_err(ectx!(try ErrorKind::Internal))?
                    );
                }
                Ok(Metrics::default())
            })
    }
}
