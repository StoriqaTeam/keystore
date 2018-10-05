use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use std::fmt::{self, Debug, Display};
use uuid::Uuid;

use super::currency::Currency;
use super::user::UserId;
use schema::keys;

#[derive(Debug, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct KeyId(Uuid);
derive_newtype_sql!(key_id, SqlUuid, KeyId, KeyId);

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

#[derive(Debug, FromSqlRow, AsExpression, Clone)]
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
    id: KeyId,
    private_key: PrivateKey,
    blockchain_address: BlockChainAddress,
    currency: Currency,
    owner_id: UserId,
    created_at: SystemTime,
    updated_at: SystemTime,
}

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "keys"]
pub struct NewKey {
    id: KeyId,
    private_key: PrivateKey,
    blockchain_address: BlockChainAddress,
    currency: Currency,
    owner_id: UserId,
}
