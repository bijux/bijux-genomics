use serde::{Deserialize, Serialize};

/// Stable BAM artifact boundary exchanged between cross-domain pipeline stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlignmentBoundary {
    pub bam_path: String,
    #[serde(default)]
    pub bai_path: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub rg_policy: Option<String>,
    #[serde(default)]
    pub aligner_meta: Option<std::collections::BTreeMap<String, String>>,
}
