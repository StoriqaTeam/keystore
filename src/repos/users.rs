use super::error::*;
use super::repo::Repo;
use diesel::Queryable;
use models::*;
use prelude::*;
use schema::users::dsl::*;

pub trait UsersRepo {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error>;
}

impl<'a> UsersRepo for Repo<'a> {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error> {
        users
            .filter(authentication_token.eq(token))
            .limit(1)
            .get_result(self.db_conn)
            .optional()
            .map_err(ectx!(ErrorKind::Internal))
    }
}
