use diesel::sql_types::Uuid as SqlUuid;
use std::fmt::{self, Debug, Display};
use uuid::Uuid;

use super::amount::Amount;
use super::currency::Currency;
use super::key::BlockchainAddress;

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

impl RawTransaction {
    pub fn new(data: String) -> Self {
        RawTransaction(data)
    }

    pub fn inner(self) -> String {
        self.0
    }
}

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
    pub utxos: Option<Vec<Utxo>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Utxo {
    pub tx_hash: String,
    pub value: Amount,
    pub index: u64,
}
