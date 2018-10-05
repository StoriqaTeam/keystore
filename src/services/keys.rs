use std::sync::Arc;

use super::auth::AuthService;
use super::error::*;
use models::*;
use prelude::*;
use repos::KeysRepo;
use diesel::pg::PgConnection;

trait KeysService {
    fn list(&self, offset: i64, limit: i64) -> Box<Future<Item = Vec<Key>, Error = Error>>;
}

pub struct KeysServiceImpl {
    db_pool: PgConnectionPool,
    auth_service: Arc<AuthService>,
    keys_repo_factory: Arc<for<'a> Fn(&'a PgConnection) -> Box<KeysRepo + 'a> + Send + Sync>,
}

impl KeysService for KeysServiceImpl {
    fn list(&self, offset: i64, limit: i64) -> Box<Future<Item = Vec<Key>, Error = Error>> {
        let keys_repo_factory = self.keys_repo_factory.clone();
        Box::new(
            self.auth_service.authenticate(offset, limit)
                .and_then(|user| {
                    let keys_repo = keys_repo_factory(user.)
                })
        )
    }
}
