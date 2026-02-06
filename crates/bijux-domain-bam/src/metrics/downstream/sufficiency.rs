use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageSufficiencyV1 {
    pub sufficient: bool,
    pub mean_coverage: f64,
    pub breadth_1x: f64,
    pub reason: String,
}

impl CoverageSufficiencyV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sufficient: false,
            mean_coverage: 0.0,
            breadth_1x: 0.0,
            reason: "unknown".to_string(),
        }
    }
}

impl Default for CoverageSufficiencyV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SexSufficiencyV1 {
    pub sufficient: bool,
    pub confidence: f64,
    pub reason: String,
}

impl SexSufficiencyV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sufficient: false,
            confidence: 0.0,
            reason: "unknown".to_string(),
        }
    }
}

impl Default for SexSufficiencyV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationSufficiencyV1 {
    pub sufficient: bool,
    pub estimate: f64,
    pub reason: String,
}

impl ContaminationSufficiencyV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sufficient: false,
            estimate: 0.0,
            reason: "unknown".to_string(),
        }
    }
}

impl Default for ContaminationSufficiencyV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HaplogroupSufficiencyV1 {
    pub sufficient: bool,
    pub min_coverage: f64,
    pub reason: String,
}

impl HaplogroupSufficiencyV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sufficient: false,
            min_coverage: 0.0,
            reason: "unknown".to_string(),
        }
    }
}

impl Default for HaplogroupSufficiencyV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KinshipSufficiencyV1 {
    pub sufficient: bool,
    pub overlap_snps: u32,
    pub reason: String,
}

impl KinshipSufficiencyV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sufficient: false,
            overlap_snps: 0,
            reason: "unknown".to_string(),
        }
    }
}

impl Default for KinshipSufficiencyV1 {
    fn default() -> Self {
        Self::empty()
    }
}
