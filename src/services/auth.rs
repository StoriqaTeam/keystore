use std::sync::Arc;

use super::error::*;
use super::ServiceFuture;
use futures::future;
use futures_cpupool::CpuPool;
use models::*;
use prelude::*;
use repos::UsersRepo;

pub trait AuthService: Send + Sync + 'static {
    fn authenticate(&self, maybe_token: Option<AuthenticationToken>) -> ServiceFuture<User>;
}

#[derive(Clone)]
pub struct AuthServiceImpl {
    db_pool: PgPool,
    thread_pool: CpuPool,
    users_repo_factory: Arc<Fn(Arc<PgPooledConnection>) -> Box<UsersRepo> + Send + Sync>,
}

impl AuthServiceImpl {
    pub fn new(
        db_pool: PgPool,
        thread_pool: CpuPool,
        users_repo_factory: Arc<Fn(Arc<PgPooledConnection>) -> Box<UsersRepo> + Send + Sync>,
    ) -> Self {
        AuthServiceImpl {
            db_pool,
            thread_pool,
            users_repo_factory,
        }
    }
}

impl AuthService for AuthServiceImpl {
    fn authenticate(&self, maybe_token: Option<AuthenticationToken>) -> ServiceFuture<User> {
        let token = match maybe_token {
            Some(t) => t,
            None => return Box::new(future::err(ErrorKind::Unauthorized.into())),
        };
        let db_pool = self.db_pool.clone();
        let users_repo_factory = self.users_repo_factory.clone();
        let token_clone = token.clone();
        let token_clone2 = token.clone();
        Box::new(self.thread_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(ectx!(ErrorSource::R2D2, ErrorKind::Internal))
                .and_then(move |conn| {
                    users_repo_factory(Arc::new(conn))
                        .find_user_by_authentication_token(token)
                        .map_err(ectx!(ErrorKind::Internal => token_clone))
                }).and_then(move |maybe_user| {
                    maybe_user.ok_or(ectx!(err ErrorContext::NoAuthToken, ErrorKind::Unauthorized => token_clone2))
                })
        }))
    }
}
