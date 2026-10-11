//! Owner: bijux-dna-runner
//! Typed Slurm requests and scheduler evidence.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const SLURM_JOB_HANDLE_SCHEMA_V1: &str = "bijux.runner.slurm_job_handle.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlurmClientPaths {
    pub sbatch: PathBuf,
    pub squeue: PathBuf,
    pub sacct: PathBuf,
    pub scancel: PathBuf,
}

impl Default for SlurmClientPaths {
    fn default() -> Self {
        Self {
            sbatch: PathBuf::from("sbatch"),
            squeue: PathBuf::from("squeue"),
            sacct: PathBuf::from("sacct"),
            scancel: PathBuf::from("scancel"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlurmSubmitRequest {
    pub run_id: String,
    pub script_path: PathBuf,
    pub working_directory: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub submitted_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlurmJobHandle {
    pub schema_version: String,
    pub run_id: String,
    pub job_id: String,
    pub script_path: PathBuf,
    pub working_directory: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub submitted_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlurmSubmissionFailure {
    pub run_id: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlurmBatchSubmission {
    pub submitted: Vec<SlurmJobHandle>,
    pub failures: Vec<SlurmSubmissionFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlurmJobState {
    Submitted,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
    Preempted,
    NodeFailure,
    OutOfMemory,
    AccountingPending,
}

impl SlurmJobState {
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Failed
                | Self::Cancelled
                | Self::Timeout
                | Self::Preempted
                | Self::NodeFailure
                | Self::OutOfMemory
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlurmEvidenceSource {
    Submission,
    Queue,
    Accounting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlurmAccounting {
    pub job_id: String,
    pub state: SlurmJobState,
    pub raw_state: String,
    pub exit_code: i32,
    pub signal: i32,
    pub elapsed_seconds: u64,
    pub total_cpu_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_rss_bytes: Option<u64>,
    pub node_list: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlurmJobObservation {
    pub job_id: String,
    pub state: SlurmJobState,
    pub source: SlurmEvidenceSource,
    pub observed_at_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accounting: Option<SlurmAccounting>,
}
