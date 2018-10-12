use models::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostKeysRequest {
    pub id: KeyId,
    pub currency: Currency,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetKeysParams {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostTransactionsRequest {
    pub id: TransactionId,
    pub from: BlockchainAddress,
    pub to: BlockchainAddress,
    pub currency: Currency,
    pub value: Amount,
    pub fee_price: Amount,
    pub nonce: Option<u64>,
    pub utxos: Option<Vec<Utxo>>,
}

impl From<PostTransactionsRequest> for UnsignedTransaction {
    fn from(req: PostTransactionsRequest) -> Self {
        let PostTransactionsRequest {
            id,
            from,
            to,
            currency,
            value,
            fee_price,
            nonce,
            utxos,
        } = req;

        UnsignedTransaction {
            id,
            from,
            to,
            currency,
            value,
            fee_price,
            nonce,
            utxos,
        }
    }
}
