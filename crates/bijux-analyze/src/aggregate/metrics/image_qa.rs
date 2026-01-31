//! Owner: bijux-analyze
//! Image QA records.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::aggregate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ImageQaOutcome {
    Pass,
    Fail(String),
}

impl ImageQaOutcome {
    #[must_use]
    pub fn status(&self) -> &'static str {
        match self {
            ImageQaOutcome::Pass => "pass",
            ImageQaOutcome::Fail(_) => "fail",
        }
    }

    #[must_use]
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            ImageQaOutcome::Pass => None,
            ImageQaOutcome::Fail(reason) => Some(reason.as_str()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageQaRecord {
    pub tool: String,
    pub stage: String,
    pub tool_version: String,
    pub image_digest: String,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub outcome: ImageQaOutcome,
}

/// Append a benchmark record as a JSONL line.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn append_image_qa_jsonl(path: &Path, record: &ImageQaRecord) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub const IMAGE_QA_SCHEMA_VERSION: i32 = 1;
pub const IMAGE_QA_INPUTS_SCHEMA_VERSION: i32 = 1;
