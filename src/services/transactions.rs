use std::sync::Arc;

use super::auth::AuthService;
use super::error::*;
use super::ServiceFuture;
use blockchain::BlockchainSigner;
use models::*;
use prelude::*;
use repos::{DbExecutor, KeysRepo};

pub trait TransactionsService: Send + Sync + 'static {
    fn sign(&self, maybe_token: Option<AuthenticationToken>, transaction: UnsignedTransaction) -> ServiceFuture<RawTransaction>;
}

pub struct TransactionsServiceImpl<E: DbExecutor> {
    auth_service: Arc<AuthService>,
    keys_repo: Arc<KeysRepo>,
    blockchain_signer: Arc<BlockchainSigner>,
    db_executor: E,
}

impl<E: DbExecutor> TransactionsServiceImpl<E> {
    pub fn new(auth_service: Arc<AuthService>, keys_repo: Arc<KeysRepo>, blockchain_signer: Arc<BlockchainSigner>, db_executor: E) -> Self {
        Self {
            auth_service,
            keys_repo,
            blockchain_signer,
            db_executor,
        }
    }
}

impl<E: DbExecutor> TransactionsService for TransactionsServiceImpl<E> {
    fn sign(&self, maybe_token: Option<AuthenticationToken>, transaction: UnsignedTransaction) -> ServiceFuture<RawTransaction> {
        let db_executor = self.db_executor.clone();
        let keys_repo = self.keys_repo.clone();
        let signer = self.blockchain_signer.clone();
        Box::new(self.auth_service.authenticate(maybe_token).and_then(move |user| {
            let user_id = user.id.clone();
            let user_id_clone = user_id.clone();
            let user_id_clone2 = user_id.clone();
            let blockchain_address = transaction.from.clone();
            let blockchain_address_clone = blockchain_address.clone();
            let currency = transaction.currency.clone();
            db_executor.execute_transaction(move || {
                keys_repo
                    .find_by_address_and_currency(user_id, currency, blockchain_address)
                    .map_err(ectx!(ErrorKind::Internal => user_id_clone))
                    .and_then(|maybe_key| {
                        maybe_key.ok_or(ectx!(err ErrorContext::NoWallet, ErrorKind::NotFound => user_id_clone2, blockchain_address_clone))
                    }).and_then(move |key| {
                        signer
                            .sign(key.private_key, transaction)
                            .map_err(ectx!(ErrorContext::SigningTransaction, ErrorKind::Internal))
                    })
            })
        }))
    }
}
