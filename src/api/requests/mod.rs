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
    pub fee_price: f64,
    pub nonce: Option<u64>,
    pub utxos: Option<Vec<Utxo>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostApproveRequest {
    pub id: TransactionId,
    pub address: BlockchainAddress,
    pub approve_address: BlockchainAddress,
    pub currency: Currency,
    pub value: Amount,
    pub fee_price: f64,
    pub nonce: u64,
}

impl From<PostApproveRequest> for ApproveInput {
    fn from(req: PostApproveRequest) -> Self {
        let PostApproveRequest {
            id,
            address,
            approve_address,
            currency,
            value,
            fee_price,
            nonce,
        } = req;
        ApproveInput {
            id,
            address,
            approve_address,
            currency,
            value,
            fee_price,
            nonce,
        }
    }
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
