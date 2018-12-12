use std::sync::Arc;

use super::auth::AuthService;
use super::error::*;
use super::ServiceFuture;
use blockchain::BlockchainService;
use models::*;
use prelude::*;
use repos::{DbExecutor, KeysRepo, UsersRepo};

pub trait TransactionsService: Send + Sync + 'static {
    fn sign(&self, maybe_token: Option<AuthenticationToken>, transaction: UnsignedTransaction) -> ServiceFuture<RawTransaction>;
    fn approve(&self, maybe_token: Option<AuthenticationToken>, input: ApproveInput) -> ServiceFuture<RawTransaction>;
}

pub struct TransactionsServiceImpl<E: DbExecutor> {
    auth_service: Arc<AuthService>,
    keys_repo: Arc<KeysRepo>,
    users_repo: Arc<UsersRepo>,
    blockchain_signer: Arc<BlockchainService>,
    stq_controller_address: BlockchainAddress,
    db_executor: E,
}

impl<E: DbExecutor> TransactionsServiceImpl<E> {
    pub fn new(
        auth_service: Arc<AuthService>,
        keys_repo: Arc<KeysRepo>,
        users_repo: Arc<UsersRepo>,
        blockchain_signer: Arc<BlockchainService>,
        stq_controller_address: BlockchainAddress,
        db_executor: E,
    ) -> Self {
        Self {
            auth_service,
            keys_repo,
            users_repo,
            blockchain_signer,
            stq_controller_address,
            db_executor,
        }
    }
}

impl<E: DbExecutor> TransactionsService for TransactionsServiceImpl<E> {
    fn sign(&self, maybe_token: Option<AuthenticationToken>, transaction: UnsignedTransaction) -> ServiceFuture<RawTransaction> {
        let db_executor = self.db_executor.clone();
        let keys_repo = self.keys_repo.clone();
        let users_repo = self.users_repo.clone();
        let signer = self.blockchain_signer.clone();
        let stq_controller_address = self.stq_controller_address.clone();
        Box::new(self.auth_service.authenticate(maybe_token).and_then(move |user| {
            let blockchain_address = match transaction.currency {
                Currency::Stq => stq_controller_address.clone(),
                _ => transaction.from.clone(),
            };
            let user_id = user.id.clone();
            let user_id_clone = user_id.clone();
            let user_id_clone2 = user_id.clone();
            let blockchain_address_clone = blockchain_address.clone();
            let currency = transaction.currency.clone();
            let currency_clone = currency.clone();
            db_executor.execute_transaction(move || {
                // stq is transferred using system account
                let user_id = match currency {
                    Currency::Stq => users_repo.find_system_user()?.ok_or(ectx!(try err ErrorContext::NoSystemUser, ErrorKind::Internal))?.id,
                    _ => user_id,
                };
                keys_repo
                    .find_by_address(user_id, blockchain_address)
                    .map_err(ectx!(ErrorKind::Internal => user_id_clone))
                    .and_then(|maybe_key| {
                        maybe_key.ok_or(ectx!(err ErrorContext::NoWallet, ErrorKind::NotFound => user_id_clone2, blockchain_address_clone, currency_clone))
                    }).and_then(move |key| {
                        signer
                            .sign(key.private_key.clone(), transaction.clone())
                            .map_err(ectx!(convert => key.private_key, transaction))
                    })
            })
        }))
    }

    fn approve(&self, maybe_token: Option<AuthenticationToken>, input: ApproveInput) -> ServiceFuture<RawTransaction> {
        if input.currency != Currency::Stq {
            return Box::new(Err(ectx!(err ErrorContext::NotSupportedCurrency, ErrorKind::MalformedInput)).into_future());
        }
        let db_executor = self.db_executor.clone();
        let keys_repo = self.keys_repo.clone();
        let signer = self.blockchain_signer.clone();
        Box::new(self.auth_service.authenticate(maybe_token).and_then(move |user| {
            let user_id = user.id.clone();
            let user_id_clone = user_id.clone();
            let user_id_clone2 = user_id.clone();
            let blockchain_address = input.address.clone();
            let blockchain_address_clone = blockchain_address.clone();
            let currency = Currency::Stq;
            db_executor.execute_transaction(move || {
                keys_repo
                    .find_by_address(user_id, blockchain_address)
                    .map_err(ectx!(ErrorKind::Internal => user_id_clone))
                    .and_then(|maybe_key| {
                        maybe_key.ok_or(
                            ectx!(err ErrorContext::NoWallet, ErrorKind::NotFound => user_id_clone2, blockchain_address_clone, currency),
                        )
                    })
                    .and_then(move |key| {
                        signer
                            .approve(key.private_key.clone(), input.clone())
                            .map_err(ectx!(convert => key.private_key, input))
                    })
            })
        }))
    }
}
