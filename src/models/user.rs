use diesel::deserialize::FromSqlRow;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::row::Row;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use diesel::Queryable;
use prelude::*;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Debug, Display};
use std::io::Write;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct UserId(Uuid);

impl FromSql<SqlUuid, Pg> for UserId {
    fn from_sql(data: Option<&[u8]>) -> deserialize::Result<Self> {
        FromSql::<SqlUuid, Pg>::from_sql(data).map(UserId)
    }
}

impl ToSql<SqlUuid, Pg> for UserId {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        ToSql::<SqlUuid, Pg>::to_sql(&self.0, out)
    }
}

#[derive(Deserialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct AuthenticationToken(String);

impl FromSql<VarChar, Pg> for AuthenticationToken {
    fn from_sql(data: Option<&[u8]>) -> deserialize::Result<Self> {
        FromSql::<VarChar, Pg>::from_sql(data).map(AuthenticationToken)
    }
}

impl ToSql<VarChar, Pg> for AuthenticationToken {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        ToSql::<VarChar, Pg>::to_sql(&self.0, out)
    }
}

const MASK: &str = "********";

impl Debug for AuthenticationToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(MASK)
    }
}

impl Display for AuthenticationToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(MASK)
    }
}

#[derive(Debug, Deserialize, Queryable, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: UserId,
    name: String,
    authentication_token: AuthenticationToken,
    created_at: SystemTime,
    updated_at: SystemTime,
}
