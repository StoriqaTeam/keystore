use std::sync::Arc;

use super::auth::AuthService;
use super::error::*;
use super::ServiceFuture;
use blockchain::KeyGenerator;
use futures_cpupool::CpuPool;
use models::*;
use prelude::*;
use repos::KeysRepo;

pub trait KeysService: Send + Sync + 'static {
    fn list(&self, maybe_token: Option<AuthenticationToken>, offset: i64, limit: i64) -> ServiceFuture<Vec<Key>>;
    fn create(&self, maybe_token: Option<AuthenticationToken>, currency: Currency, id: KeyId) -> ServiceFuture<Key>;
}

pub struct KeysServiceImpl {
    db_pool: PgPool,
    auth_service: Arc<AuthService>,
    thread_pool: CpuPool,
    keys_repo_factory: Arc<Fn(Arc<PgPooledConnection>) -> Box<KeysRepo> + Send + Sync>,
    key_generator: Arc<KeyGenerator>,
}

impl KeysServiceImpl {
    pub fn new(
        db_pool: PgPool,
        auth_service: Arc<AuthService>,
        thread_pool: CpuPool,
        keys_repo_factory: Arc<Fn(Arc<PgPooledConnection>) -> Box<KeysRepo> + Send + Sync>,
        key_generator: Arc<KeyGenerator>,
    ) -> Self {
        Self {
            db_pool,
            auth_service,
            thread_pool,
            keys_repo_factory,
            key_generator,
        }
    }
}

impl KeysService for KeysServiceImpl {
    fn list(&self, maybe_token: Option<AuthenticationToken>, offset: i64, limit: i64) -> ServiceFuture<Vec<Key>> {
        let keys_repo_factory = self.keys_repo_factory.clone();
        let thread_pool = self.thread_pool.clone();
        let db_pool = self.db_pool.clone();
        Box::new(self.auth_service.authenticate(maybe_token).and_then(move |user| {
            thread_pool.spawn_fn(move || {
                let user_id = user.id;
                let user_id_clone = user_id.clone();
                db_pool
                    .get()
                    .map_err(ectx!(ErrorSource::R2D2, ErrorKind::Internal))
                    .and_then(|conn| {
                        keys_repo_factory(Arc::new(conn))
                            .list(user_id, offset, limit)
                            .map_err(ectx!(ErrorKind::Internal => user_id_clone, offset, limit))
                    })
            })
        }))
    }

    fn create(&self, maybe_token: Option<AuthenticationToken>, currency: Currency, id: KeyId) -> ServiceFuture<Key> {
        let keys_repo_factory = self.keys_repo_factory.clone();
        let thread_pool = self.thread_pool.clone();
        let db_pool = self.db_pool.clone();
        let key_generator = self.key_generator.clone();
        let id_clone = id.clone();
        Box::new(self.auth_service.authenticate(maybe_token).and_then(move |user| {
            thread_pool.spawn_fn(move || {
                let owner_id = user.id;
                let owner_id_clone = owner_id.clone();
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
                        keys_repo_factory(Arc::new(conn))
                            .create(new_key)
                            .map_err(ectx!(ErrorKind::Internal => owner_id_clone, currency, id_clone))
                    })
            })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain::*;
    use repos::*;
    use services::*;
    use {create_db_pool, get_config};

    #[test]
    fn test_create() {
        // let db_pool = create_db_pool(&get_config());
        // let users_repo = UsersRepoMock::new();
        // let new_user = NewUser::default();
        // let token = new_user.authentication_token.clone();
        // let _ = users_repo.create(new_user);
        // let keys_repo_mock = KeysRepoMock::new();
        // let auth_service = Arc::new(AuthServiceMock::new(vec![token.clone()]));
        // let thread_pool = CpuPool::new(1);
        // let key_generator = Arc::new(KeyGeneratorMock);
        // let keys_repo_factory = Arc::new(move |_| -> Box<KeysRepo> { Box::new(keys_repo_mock.clone()) });
        // let keys_service = KeysServiceImpl::new(db_pool, auth_service, thread_pool, keys_repo_factory, key_generator);
    }
}

// db_pool: PgPool,
// auth_service: Arc<AuthService>,
// thread_pool: CpuPool,
// keys_repo_factory: Arc<for<'a> Fn(&'a PgConnection) -> Box<KeysRepo + 'a> + Send + Sync>,
// key_generator: Arc<KeyGenerator>,
