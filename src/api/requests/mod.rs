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
