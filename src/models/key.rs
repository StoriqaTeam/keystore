use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use std::fmt::{self, Debug, Display};
use uuid::Uuid;

use super::currency::Currency;
use super::user::UserId;
use schema::keys;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct KeyId(Uuid);
derive_newtype_sql!(key_id, SqlUuid, KeyId, KeyId);

impl Default for KeyId {
    fn default() -> Self {
        KeyId(Uuid::new_v4())
    }
}

#[derive(FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct PrivateKey(String);
derive_newtype_sql!(private_key_id, VarChar, PrivateKey, PrivateKey);
mask_logs!(PrivateKey);

impl PrivateKey {
    pub fn new(data: String) -> Self {
        PrivateKey(data)
    }
}

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct BlockChainAddress(String);
derive_newtype_sql!(blockchain_address, VarChar, BlockChainAddress, BlockChainAddress);

impl BlockChainAddress {
    pub fn new(data: String) -> Self {
        BlockChainAddress(data)
    }
}

#[derive(Debug, Queryable, Clone)]
pub struct Key {
    pub id: KeyId,
    pub private_key: PrivateKey,
    pub blockchain_address: BlockChainAddress,
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
    pub blockchain_address: BlockChainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
}
