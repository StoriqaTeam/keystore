use super::error::Error;
use super::repo::Repo;
use diesel;
use diesel::pg::PgConnection;
use models::*;
use schema::users::dsl::*;

pub trait UsersRepo {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<AuthenticationToken>, Error>;
}

impl UsersRepo for Repo {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<AuthenticationToken>, Error> {
        let query = users.filter(authentication_token.eq(token)).first(self.db_conn)
    }
}
