use super::{KeysRepo, KeysRepoImpl, UsersRepo, UsersRepoImpl};
use diesel::pg::PgConnection;

pub fn create_users_repo<'a>(db_conn: &'a PgConnection) -> Box<UsersRepo + 'a> {
    Box::new(UsersRepoImpl::new(db_conn))
}

pub fn create_keys_repo<'a>(db_conn: &'a PgConnection) -> Box<KeysRepo + 'a> {
    Box::new(KeysRepoImpl::new(db_conn))
}
