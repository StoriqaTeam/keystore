use std::sync::Arc;

use super::auth::AuthService;
use super::error::*;
use super::ServiceFuture;
use blockchain::KeyGenerator;
use models::*;
use prelude::*;
use repos::{DbExecutor, KeysRepo};

pub trait KeysService: Send + Sync + 'static {
    fn list(&self, maybe_token: Option<AuthenticationToken>, offset: i64, limit: i64) -> ServiceFuture<Vec<Key>>;
    fn create(&self, maybe_token: Option<AuthenticationToken>, currency: Currency, id: KeyId) -> ServiceFuture<Key>;
}

pub struct KeysServiceImpl<E: DbExecutor> {
    auth_service: Arc<AuthService>,
    key_generator: Arc<KeyGenerator>,
    keys_repo: Arc<KeysRepo>,
    db_executor: E,
}

impl<E: DbExecutor> KeysServiceImpl<E> {
    pub fn new(auth_service: Arc<AuthService>, key_generator: Arc<KeyGenerator>, keys_repo: Arc<KeysRepo>, db_executor: E) -> Self {
        Self {
            auth_service,
            key_generator,
            keys_repo,
            db_executor,
        }
    }
}

impl<E: DbExecutor> KeysService for KeysServiceImpl<E> {
    fn list(&self, maybe_token: Option<AuthenticationToken>, offset: i64, limit: i64) -> ServiceFuture<Vec<Key>> {
        let db_executor = self.db_executor.clone();
        let keys_repo = self.keys_repo.clone();
        Box::new(self.auth_service.authenticate(maybe_token).and_then(move |user| {
            let user_id = user.id.clone();
            let user_id_clone = user_id.clone();
            db_executor.execute_transaction(move || {
                keys_repo
                    .list(user_id, offset, limit)
                    .map_err(ectx!(ErrorKind::Internal => user_id_clone, offset, limit))
            })
        }))
    }

    fn create(&self, maybe_token: Option<AuthenticationToken>, currency: Currency, id: KeyId) -> ServiceFuture<Key> {
        let db_executor = self.db_executor.clone();
        let keys_repo = self.keys_repo.clone();
        let id_clone = id.clone();
        let key_generator = self.key_generator.clone();
        Box::new(self.auth_service.authenticate(maybe_token).and_then(move |user| {
            let owner_id = user.id;
            let owner_id_clone = owner_id.clone();
            db_executor.execute(move || {
                let (private_key, blockchain_address) = key_generator.generate_key(currency).map_err(ectx!(try ErrorKind::Internal))?;
                let new_key = NewKey {
                    id,
                    currency,
                    owner_id,
                    private_key,
                    blockchain_address,
                };
                keys_repo
                    .create(new_key)
                    .map_err(ectx!(convert => owner_id_clone, currency, id_clone))
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
    use tokio_core::reactor::Core;

    #[test]
    fn test_create() {
        let new_user = NewUser::default();
        let token = new_user.authentication_token.clone();
        let auth_service = Arc::new(AuthServiceMock::new(vec![token.clone()]));
        let key_generator = Arc::new(KeyGeneratorMock);
        let keys_repo = Arc::new(KeysRepoMock::new());
        let db_executor = DbExecutorMock::new();
        let keys_service = KeysServiceImpl::new(auth_service, key_generator, keys_repo, db_executor);
        let mut core = Core::new().unwrap();

        // creates with right token
        let keys_count = core.run(keys_service.list(Some(token.clone()), 0, 100)).unwrap().len();
        assert_eq!(keys_count, 0);
        let key_id = KeyId::default();
        let res = core.run(keys_service.create(Some(token.clone()), Currency::Eth, key_id.clone()));
        assert_eq!(res.is_ok(), true);
        let keys = core.run(keys_service.list(Some(token.clone()), 0, 100)).unwrap();
        assert_eq!(keys[0].id, key_id.clone());
        assert_eq!(keys.len(), 1);

        // doesn't create with wrong token
        let auth_token2 = NewUser::default().authentication_token;
        let key_id = KeyId::default();
        let res = core.run(keys_service.create(Some(auth_token2.clone()), Currency::Eth, key_id.clone()));
        assert_eq!(res.is_err(), true);
        let keys_count = core.run(keys_service.list(Some(token.clone()), 0, 100)).unwrap().len();
        assert_eq!(keys_count, 1);

        // doesn't create with no token
        let res = core.run(keys_service.create(None, Currency::Eth, key_id.clone()));
        assert_eq!(res.is_err(), true);
        let keys_count = core.run(keys_service.list(Some(token.clone()), 0, 100)).unwrap().len();
        assert_eq!(keys_count, 1);
    }

    #[test]
    fn test_list() {
        let new_user = NewUser::default();
        let token = new_user.authentication_token.clone();
        let auth_service = Arc::new(AuthServiceMock::new(vec![token.clone()]));
        let key_generator = Arc::new(KeyGeneratorMock);
        let keys_repo = Arc::new(KeysRepoMock::new());
        let db_executor = DbExecutorMock::new();
        let keys_service = KeysServiceImpl::new(auth_service, key_generator, keys_repo, db_executor);
        let mut core = Core::new().unwrap();

        // lists with right token
        let keys_count = core.run(keys_service.list(Some(token.clone()), 0, 100)).unwrap().len();
        assert_eq!(keys_count, 0);
        let key_id = KeyId::default();
        let res = core.run(keys_service.create(Some(token.clone()), Currency::Eth, key_id.clone()));
        assert_eq!(res.is_ok(), true);
        let keys = core.run(keys_service.list(Some(token.clone()), 0, 100)).unwrap();
        assert_eq!(keys[0].id, key_id.clone());
        assert_eq!(keys.len(), 1);

        // doesn't list with wrong token
        let auth_token2 = NewUser::default().authentication_token;
        let res = core.run(keys_service.list(Some(auth_token2.clone()), 0, 100));
        assert_eq!(res.is_err(), true);

        // doesn't list with no token
        let res = core.run(keys_service.list(None, 0, 100));
        assert_eq!(res.is_err(), true);
    }
}
