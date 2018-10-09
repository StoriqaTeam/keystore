use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use std::fmt::{self, Debug, Display};
use uuid::Uuid;

use super::amount::Amount;
use super::currency::Currency;
use super::key::BlockchainAddress;
use super::user::UserId;
use schema::keys;

#[derive(Serialize, Deserialize, PartialEq, Eq, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct TransactionId(Uuid);
derive_newtype_sql!(key_id, SqlUuid, TransactionId, TransactionId);

impl Default for TransactionId {
    fn default() -> Self {
        TransactionId(Uuid::new_v4())
    }
}

impl Debug for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RawTransaction(String);

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedTransaction {
    pub id: TransactionId,
    pub from: BlockchainAddress,
    pub to: BlockchainAddress,
    pub currency: Currency,
    pub value: Amount,
    pub fee_price: Amount,
    pub nonce: Option<u64>,
}
