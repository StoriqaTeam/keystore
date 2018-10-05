use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use uuid::Uuid;

use super::currency::Currency;
use super::user::UserId;

#[derive(Debug, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct KeyId(Uuid);
derive_newtype_sql!(key_id, SqlUuid, KeyId, KeyId);

#[derive(FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct PrivateKey(String);
derive_newtype_sql!(private_key_id, VarChar, PrivateKey, PrivateKey);
mask_logs!(PrivateKey);

#[derive(Debug, FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct BlockChainAddress(String);
derive_newtype_sql!(blockchain_address, VarChar, BlockChainAddress, BlockChainAddress);

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
