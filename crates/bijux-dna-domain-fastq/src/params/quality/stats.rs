use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const STATS_SCHEMA_VERSION: &str = "bijux.fastq.params.stats.v1";
pub const READ_LENGTH_PROFILE_SCHEMA_VERSION: &str = "bijux.fastq.params.read_length_profile.v1";
pub const OVERREPRESENTED_PROFILE_SCHEMA_VERSION: &str =
    "bijux.fastq.params.overrepresented_profile.v1";
pub const ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SCHEMA_VERSION: &str =
    "bijux.fastq.params.estimate_library_complexity_prealign.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FastqStatsParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
}

impl FastqStatsParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FastqEstimateLibraryComplexityPrealignParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub complexity_policy: String,
    pub estimate_method: String,
    pub advisory_only: bool,
    pub modifies_reads: bool,
    pub kmer_size: Option<u32>,
}

impl FastqEstimateLibraryComplexityPrealignParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.complexity_policy.trim().is_empty() {
            missing.push("complexity_policy");
        }
        if self.estimate_method.trim().is_empty() {
            missing.push("estimate_method");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "complexity_policy": self.complexity_policy,
            "estimate_method": self.estimate_method,
            "advisory_only": self.advisory_only,
            "modifies_reads": self.modifies_reads,
            "kmer_size": self.kmer_size,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FastqReadLengthProfileParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub histogram_bins: u32,
}

impl FastqReadLengthProfileParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.histogram_bins == 0 {
            missing.push("histogram_bins");
        }
        missing
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FastqOverrepresentedProfileParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub top_k: u32,
}

impl FastqOverrepresentedProfileParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.top_k == 0 {
            missing.push("top_k");
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FastqEstimateLibraryComplexityPrealignParams, FastqOverrepresentedProfileParams,
        FastqReadLengthProfileParams, ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SCHEMA_VERSION,
        OVERREPRESENTED_PROFILE_SCHEMA_VERSION, READ_LENGTH_PROFILE_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

    #[test]
    fn estimate_library_complexity_params_require_declared_policy_and_method() {
        let params = FastqEstimateLibraryComplexityPrealignParams {
            schema_version: ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SCHEMA_VERSION.to_string(),
            paired_mode: PairedMode::PairedEnd,
            complexity_policy: "prealign_kmer".to_string(),
            estimate_method: "kmer_redundancy".to_string(),
            advisory_only: true,
            modifies_reads: false,
            kmer_size: Some(21),
        };

        assert!(params.missing_required_fields().is_empty());
    }

    #[test]
    fn overrepresented_profile_params_require_threads_and_top_k() {
        let params = FastqOverrepresentedProfileParams {
            schema_version: OVERREPRESENTED_PROFILE_SCHEMA_VERSION.to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 4,
            top_k: 25,
        };

        assert!(params.missing_required_fields().is_empty());
    }

    #[test]
    fn read_length_profile_params_require_threads_and_histogram_bins() {
        let params = FastqReadLengthProfileParams {
            schema_version: READ_LENGTH_PROFILE_SCHEMA_VERSION.to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            histogram_bins: 64,
        };

        assert!(params.missing_required_fields().is_empty());
    }
}
