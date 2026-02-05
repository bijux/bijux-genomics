// split to keep module size manageable

pub mod alignment;
pub mod contract;
pub mod domain;
pub mod errors;
pub mod events;
pub mod explain;
pub mod hashing;
pub mod input_assessment;
pub mod invariants;
pub mod measure;
pub mod metrics;
pub mod metrics_registry;
pub mod observability;
pub mod plan;
pub mod run_index;
pub mod scientific_provenance;
pub mod selection;
pub mod telemetry;
pub use contract::validate_execution_outputs;
pub use contract::ExecutionManifest;
pub use contract::{RunRecordV1, StageExecutionRecordV1};
pub use explain::{ExplainExclusion, ExplainPlan, PlanExplainStageV1, PlanExplainV1};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;

include!("lib/exports.rs");
