use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const STATS_SCHEMA_VERSION: &str = "bijux.fastq.params.stats.v1";
pub const READ_LENGTH_PROFILE_SCHEMA_VERSION: &str =
    "bijux.fastq.params.read_length_profile.v1";
pub const OVERREPRESENTED_PROFILE_SCHEMA_VERSION: &str =
    "bijux.fastq.params.overrepresented_profile.v1";

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
        FastqOverrepresentedProfileParams, FastqReadLengthProfileParams,
        OVERREPRESENTED_PROFILE_SCHEMA_VERSION, READ_LENGTH_PROFILE_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

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
