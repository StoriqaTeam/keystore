use schema::users;
use std::fmt;
use std::fmt::{Debug, Display};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Clone)]
pub struct UserId(Uuid);

#[derive(Deserialize, Queryable, Clone)]
pub struct AuthenticationToken(String);

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
