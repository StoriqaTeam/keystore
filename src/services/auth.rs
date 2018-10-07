use std::sync::Arc;

use super::error::*;
use super::ServiceFuture;
use futures::future;
use futures_cpupool::CpuPool;
use models::*;
use prelude::*;
use repos::{ErrorKind as DieselErrorKind, UsersRepo};

pub trait AuthService: Send + Sync + 'static {
    fn authenticate(&self, maybe_token: Option<AuthenticationToken>) -> ServiceFuture<User>;
}

#[derive(Clone)]
pub struct AuthServiceImpl<U: UsersRepo> {
    db_pool: PgPool,
    thread_pool: CpuPool,
    users_repo: U,
}

impl<U: UsersRepo> AuthServiceImpl<U> {
    pub fn new(db_pool: PgPool, thread_pool: CpuPool, users_repo: U) -> Self {
        AuthServiceImpl {
            db_pool,
            thread_pool,
            users_repo,
        }
    }
}

impl<U: UsersRepo> AuthService for AuthServiceImpl<U> {
    fn authenticate(&self, maybe_token: Option<AuthenticationToken>) -> ServiceFuture<User> {
        let token = match maybe_token {
            Some(t) => t,
            None => return Box::new(future::err(ErrorKind::Unauthorized.into())),
        };
        let users_repo = self.users_repo.clone();
        let token_clone = token.clone();
        let token_clone2 = token.clone();
        Box::new(
            self.users_repo
                .execute(move || {
                    users_repo
                        .find_user_by_authentication_token(token)
                        // .map_err(ectx!(ErrorKind::Internal => token_clone))
                        .and_then(move |maybe_user| {
                            // maybe_user.ok_or(ectx!(err ErrorContext::NoAuthToken, ErrorKind::Unauthorized => token_clone2))
                            maybe_user.ok_or(ectx!(err DieselErrorKind::Internal, DieselErrorKind::Internal => token_clone2))
                        })
                }).map_err(ectx!(ErrorKind::Internal => token_clone)),
        )
    }
}
