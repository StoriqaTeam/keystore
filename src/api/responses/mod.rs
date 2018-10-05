use models::*;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostKeysResponse {
    pub id: KeyId,
    pub currency: Currency,
    pub blockchain_address: BlockChainAddress,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyResponse {
    pub currency: Currency,
    pub blockchain_address: BlockChainAddress,
}
