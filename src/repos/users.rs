use std::sync::Arc;

use super::error::*;
use super::repo::Repo;
use diesel;
use futures_cpupool::CpuPool;
use models::*;
use prelude::*;
use schema::users::dsl::*;

pub trait UsersRepo: Repo {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error>;
    fn create(&self, payload: NewUser) -> Result<User, Error>;
}

#[derive(Clone)]
pub struct UsersRepoImpl {
    db_pool: PgPool,
    thread_pool: CpuPool,
}

impl UsersRepoImpl {
    pub fn new(db_pool: PgPool, thread_pool: CpuPool) -> Self {
        UsersRepoImpl { db_pool, thread_pool }
    }
}

impl Repo for UsersRepoImpl {
    fn get_db_pool(&self) -> PgPool {
        self.db_pool.clone()
    }
    fn get_db_thread_pool(&self) -> CpuPool {
        self.thread_pool.clone()
    }
}

impl<'a> UsersRepo for UsersRepoImpl {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error> {
        self.with_tls_connection(|conn| {
            users
                .filter(authentication_token.eq(token))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(ectx!(ErrorKind::Internal))
        })
    }

    fn create(&self, payload: NewUser) -> Result<User, Error> {
        self.with_tls_connection(|conn| {
            diesel::insert_into(users)
                .values(payload.clone())
                .get_result::<User>(conn)
                .map_err(ectx!(ErrorKind::Internal => payload))
        })
    }
}
