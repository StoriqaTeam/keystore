#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename = "camelCase")]
pub struct Metrics {
    total_keys: u64,
    failed_derivations_count: u64,
}
