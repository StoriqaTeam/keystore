use std::fmt;
use std::fmt::{Debug, Display};
use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct UserId(Uuid);
derive_newtype_sql!(user_id, SqlUuid, UserId, UserId);

#[derive(Deserialize, FromSqlRow, AsExpression, PartialEq, Eq, Hash, Clone)]
#[sql_type = "VarChar"]
pub struct AuthenticationToken(String);
derive_newtype_sql!(authentication_token, VarChar, AuthenticationToken, AuthenticationToken);
mask_logs!(AuthenticationToken);

impl AuthenticationToken {
    pub fn new(data: String) -> Self {
        AuthenticationToken(data)
    }
}

impl Default for AuthenticationToken {
    fn default() -> Self {
        AuthenticationToken(format!("{}", Uuid::new_v4()))
    }
}

#[derive(Debug, Deserialize, Queryable, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub authentication_token: AuthenticationToken,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: UserId(Uuid::new_v4()),
            name: "Anonymous".to_string(),
            authentication_token: Default::default(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}
