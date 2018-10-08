use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use models::*;
use prelude::*;
use schema::keys::dsl::*;

pub trait KeysRepo: Send + Sync + 'static {
    fn list(&self, current_user_id: UserId, offset: i64, limit: i64) -> Result<Vec<Key>, Error>;
    fn create(&self, payload: NewKey) -> Result<Key, Error>;
}

pub struct KeysRepoImpl;

impl KeysRepo for KeysRepoImpl {
    fn list(&self, current_user_id: UserId, offset: i64, limit: i64) -> Result<Vec<Key>, Error> {
        with_tls_connection(|conn| {
            keys.filter(owner_id.eq(current_user_id))
                .offset(offset)
                .limit(limit)
                .get_results(conn)
                .map_err(ectx!(ErrorKind::Internal))
        })
    }

    fn create(&self, payload: NewKey) -> Result<Key, Error> {
        with_tls_connection(|conn| {
            diesel::insert_into(keys)
                .values(payload.clone())
                .get_result::<Key>(conn)
                .map_err(ectx!(ErrorKind::Internal => payload))
        })
    }
}
