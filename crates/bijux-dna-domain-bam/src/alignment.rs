use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentBoundary {
    pub bam_path: String,
    pub bai_path: Option<String>,
    pub reference: Option<String>,
    pub rg_policy: Option<String>,
    pub aligner_meta: Option<std::collections::BTreeMap<String, String>>,
}
