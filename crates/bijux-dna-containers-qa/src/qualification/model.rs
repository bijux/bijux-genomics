use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationProbeKind {
    Version,
    Help,
    Real,
    Negative,
}

impl QualificationProbeKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Help => "help",
            Self::Real => "real",
            Self::Negative => "negative",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationProbePlan {
    pub kind: QualificationProbeKind,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationInputBind {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualificationMetricPlan {
    pub id: String,
    pub path: PathBuf,
    pub json_pointer: String,
    pub operator: String,
    pub expected: Value,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerQualificationPlan {
    pub pipeline_schema: String,
    pub image_path: PathBuf,
    pub image_sha256: String,
    pub inputs: Value,
    pub input_binds: Vec<QualificationInputBind>,
    pub probes: Vec<QualificationProbePlan>,
    pub output_mount: PathBuf,
    pub required_files: Vec<PathBuf>,
    pub version_regex: String,
    pub metrics: Vec<QualificationMetricPlan>,
    pub execution: Value,
    pub dependencies: Value,
    pub parameters: Value,
    pub parameter_files: Vec<PathBuf>,
    pub interface: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationProbeObservation {
    pub kind: QualificationProbeKind,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationFailureClass {
    VersionExit,
    VersionMismatch,
    HelpExit,
    RealExit,
    NegativeExit,
    RequiredOutput,
    MetricRejected,
    MetricUnreadable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationFailure {
    pub class: QualificationFailureClass,
    pub subject: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationOutputEvidence {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationMetricEvidence {
    pub id: String,
    pub path: PathBuf,
    pub json_pointer: String,
    pub operator: String,
    pub expected: Value,
    pub observed: Value,
    pub unit: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationAssessment {
    pub failures: Vec<QualificationFailure>,
    pub outputs: Vec<QualificationOutputEvidence>,
    pub metrics: Vec<QualificationMetricEvidence>,
}

impl ContainerQualificationAssessment {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }

    #[must_use]
    pub fn failure_messages(&self) -> Vec<String> {
        self.failures.iter().map(|failure| failure.detail.clone()).collect()
    }
}
