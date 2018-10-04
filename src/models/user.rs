use std::fmt;
use std::fmt::{Debug, Display};
use std::time::SystemTime;
use uuid::Uuid;

pub struct UserId(Uuid);

#[derive(Deserialize, Clone)]
pub struct AuthorizationToken(String);

const MASK: &str = "********";

impl Debug for AuthorizationToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(MASK)
    }
}

impl Display for AuthorizationToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(MASK)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[table_name = "users"]
pub struct User {
    id: UserId,
    name: String,
    authorization_token: AuthorizationToken,
    created_at: SystemTime,
    updated_at: SystemTime,
}
