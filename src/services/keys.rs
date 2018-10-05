use std::sync::Arc;

use super::auth::AuthService;
use super::error::*;
use blockchain::KeyGenerator;
use diesel::pg::PgConnection;
use futures_cpupool::CpuPool;
use models::*;
use prelude::*;
use repos::KeysRepo;

pub trait KeysService: Send + Sync + 'static {
    fn list(&self, token: AuthenticationToken, offset: i64, limit: i64) -> Box<Future<Item = Vec<Key>, Error = Error>>;
    fn create(&self, token: AuthenticationToken, currency: Currency, id: KeyId) -> Box<Future<Item = Key, Error = Error>>;
}

pub struct KeysServiceImpl {
    db_pool: PgConnectionPool,
    auth_service: Arc<AuthService>,
    thread_pool: CpuPool,
    keys_repo_factory: Arc<for<'a> Fn(&'a PgConnection) -> Box<KeysRepo + 'a> + Send + Sync>,
    key_generator: Arc<KeyGenerator>,
}

impl KeysService for KeysServiceImpl {
    fn list(&self, token: AuthenticationToken, offset: i64, limit: i64) -> Box<Future<Item = Vec<Key>, Error = Error>> {
        let keys_repo_factory = self.keys_repo_factory.clone();
        let thread_pool = self.thread_pool.clone();
        let db_pool = self.db_pool.clone();
        Box::new(self.auth_service.authenticate(token).and_then(move |user| {
            thread_pool.spawn_fn(move || {
                let user_id = user.id;
                db_pool
                    .get()
                    .map_err(ectx!(ErrorSource::R2D2, ErrorKind::Internal))
                    .and_then(|conn| {
                        keys_repo_factory(&conn)
                            .list(user_id, offset, limit)
                            .map_err(ectx!(ErrorKind::Internal => user_id, offset, limit))
                    })
            })
        }))
    }

    fn create(&self, token: AuthenticationToken, currency: Currency, id: KeyId) -> Box<Future<Item = Key, Error = Error>> {
        let keys_repo_factory = self.keys_repo_factory.clone();
        let thread_pool = self.thread_pool.clone();
        let db_pool = self.db_pool.clone();
        let key_generator = self.key_generator.clone();
        Box::new(self.auth_service.authenticate(token).and_then(move |user| {
            thread_pool.spawn_fn(move || {
                let owner_id = user.id;
                db_pool
                    .get()
                    .map_err(ectx!(ErrorSource::R2D2, ErrorKind::Internal))
                    .and_then(|conn| {
                        let (private_key, blockchain_address) = key_generator.generate_key(currency);
                        let new_key = NewKey {
                            id,
                            currency,
                            owner_id,
                            private_key,
                            blockchain_address,
                        };
                        keys_repo_factory(&conn)
                            .create(new_key)
                            .map_err(ectx!(ErrorKind::Internal => owner_id, currency, id))
                    })
            })
        }))
    }
}
