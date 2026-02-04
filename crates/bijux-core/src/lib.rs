// split to keep module size manageable

pub mod alignment;
pub mod domain;
pub mod execution_plan;
pub mod scientific_provenance;
pub mod stage_plugin;
pub mod events;
pub mod hashing;
pub mod errors;
pub mod input_assessment;
pub mod invariants;
pub mod measure;
pub mod metrics;
pub mod metrics_registry;
pub mod observability;
pub mod run_index;
pub mod selection;
pub mod stage_plan;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;

include!("lib/exports.rs");
