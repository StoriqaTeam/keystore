use std::error::Error as StdError;
use std::str::FromStr;
use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use ethereum_types::H160;
use std::fmt::{self, Debug, Display};
use uuid::Uuid;

use super::currency::Currency;
use super::user::UserId;
use blockchain::{Error as BlockchainError, ErrorContext as BlockchainErrorContext, ErrorKind as BlockchainErrorKind};
use prelude::*;
use schema::keys;

#[derive(Serialize, Deserialize, PartialEq, Eq, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct KeyId(Uuid);
derive_newtype_sql!(key_id, SqlUuid, KeyId, KeyId);

impl Default for KeyId {
    fn default() -> Self {
        KeyId(Uuid::new_v4())
    }
}

impl Debug for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

/// Hex encoded private key
#[derive(FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct PrivateKey(String);
derive_newtype_sql!(private_key_id, VarChar, PrivateKey, PrivateKey);
mask_logs!(PrivateKey);

impl PrivateKey {
    pub fn new(data: String) -> Self {
        PrivateKey(data)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Hex encoded blockchain address
#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct BlockchainAddress(String);
derive_newtype_sql!(blockchain_address, VarChar, BlockchainAddress, BlockchainAddress);

impl BlockchainAddress {
    pub fn new(data: String) -> Self {
        BlockchainAddress(data)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Queryable, Clone)]
pub struct Key {
    pub id: KeyId,
    pub private_key: PrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "keys"]
pub struct NewKey {
    pub id: KeyId,
    pub private_key: PrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
}
