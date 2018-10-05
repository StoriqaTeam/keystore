use std::sync::Arc;

use super::error::*;
use diesel::pg::PgConnection;
use futures_cpupool::CpuPool;
use models::*;
use prelude::*;
use repos::UsersRepo;

pub trait AuthService: Send + Sync + 'static {
    fn authenticate(&self, token: AuthenticationToken) -> Box<Future<Item = User, Error = Error>>;
}

#[derive(Clone)]
pub struct AuthServiceImpl {
    db_pool: PgConnectionPool,
    thread_pool: CpuPool,
    users_repo_factory: Arc<for<'a> Fn(&'a PgConnection) -> Box<UsersRepo + 'a> + Send + Sync>,
}

impl AuthService for AuthServiceImpl {
    fn authenticate(&self, token: AuthenticationToken) -> Box<Future<Item = User, Error = Error>> {
        let db_pool = self.db_pool.clone();
        let users_repo_factory = self.users_repo_factory.clone();
        let token_clone = token.clone();
        let token_clone2 = token.clone();
        Box::new(self.thread_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(ectx!(ErrorSource::R2D2, ErrorKind::Internal))
                .and_then(move |conn| {
                    users_repo_factory(&conn)
                        .find_user_by_authentication_token(token)
                        .map_err(ectx!(ErrorKind::Internal => token_clone))
                }).and_then(move |maybe_user| {
                    let e: Error = ErrorKind::Unauthorized.into();
                    maybe_user.ok_or(ectx!(err e => token_clone2))
                })
        }))
    }
}
