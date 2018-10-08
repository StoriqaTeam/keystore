use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use models::*;
use prelude::*;
use schema::users::dsl::*;

pub trait UsersRepo: Send + Sync + 'static {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error>;
    fn create(&self, payload: NewUser) -> Result<User, Error>;
}

#[derive(Clone)]
pub struct UsersRepoImpl;

impl<'a> UsersRepo for UsersRepoImpl {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error> {
        with_tls_connection(|conn| {
            users
                .filter(authentication_token.eq(token))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(ectx!(ErrorKind::Internal))
        })
    }

    fn create(&self, payload: NewUser) -> Result<User, Error> {
        let payload_clone = payload.clone();
        with_tls_connection(|conn| {
            diesel::insert_into(users)
                .values(payload.clone())
                .get_result::<User>(conn)
                .map_err(move |e| {
                    let kind = ErrorKind::from_diesel(&e);
                    ectx!(err e, kind => payload_clone)
                })
        })
    }
}
