use models::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostKeysRequest {
    pub currency: Currency,
}
