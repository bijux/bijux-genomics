// split to keep module size manageable

pub mod alignment;
pub mod domain;
pub mod errors;
pub mod events;
pub mod execution_contract;
pub mod execution_manifest;
pub mod execution_plan;
pub mod explain;
pub mod hashing;
pub mod input_assessment;
pub mod invariants;
pub mod measure;
pub mod metrics;
pub mod metrics_registry;
pub mod observability;
pub mod run_index;
pub mod run_record;
pub mod scientific_provenance;
pub mod selection;
pub mod stage_plan;
pub mod stage_plugin;

pub use execution_contract::validate_execution_outputs;
pub use execution_manifest::ExecutionManifest;
pub use explain::{ExplainExclusion, ExplainPlan, PlanExplainStageV1, PlanExplainV1};
pub use run_record::{RunRecordV1, StageExecutionRecordV1};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;

include!("lib/exports.rs");
