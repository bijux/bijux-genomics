use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub mean_q: f64,
    #[serde(default)]
    pub validated_inputs: Option<u64>,
    #[serde(default)]
    pub validated_pairs: Option<u64>,
    #[serde(default)]
    pub pair_sync_checked: Option<bool>,
    #[serde(default)]
    pub pair_sync_pass: Option<bool>,
    #[serde(default)]
    pub pair_count_match: Option<bool>,
    #[serde(default)]
    pub strict_pass: Option<bool>,
    #[serde(default)]
    pub failure_class: Option<String>,
}
