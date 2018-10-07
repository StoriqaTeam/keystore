use std::sync::Arc;

use diesel;

use super::error::*;
use models::*;
use prelude::*;
use schema::keys::dsl::*;

pub trait KeysRepo {
    fn list(&self, current_user_id: UserId, offset: i64, limit: i64) -> Result<Vec<Key>, Error>;
    fn create(&self, payload: NewKey) -> Result<Key, Error>;
}

pub struct KeysRepoImpl {
    db_conn: Arc<PgPooledConnection>,
}

impl<'a> KeysRepoImpl {
    pub fn new(db_conn: Arc<PgPooledConnection>) -> Self {
        KeysRepoImpl { db_conn }
    }
}

impl<'a> KeysRepo for KeysRepoImpl {
    fn list(&self, current_user_id: UserId, offset: i64, limit: i64) -> Result<Vec<Key>, Error> {
        keys.filter(owner_id.eq(current_user_id))
            .offset(offset)
            .limit(limit)
            .get_results(&*self.db_conn)
            .map_err(ectx!(ErrorKind::Internal))
    }

    fn create(&self, payload: NewKey) -> Result<Key, Error> {
        diesel::insert_into(keys)
            .values(payload.clone())
            .get_result::<Key>(&*self.db_conn)
            .map_err(ectx!(ErrorKind::Internal => payload))
    }
}
