use models::*;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostKeysResponse {
    pub currency: Currency,
    pub address: Address,
}
