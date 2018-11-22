#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename = "camelCase")]
pub struct Metrics {
    pub total_keys: u64,
    pub failed_derivations_count: u64,
}
