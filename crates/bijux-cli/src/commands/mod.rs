#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tracing::{info, warn};

use bijux_api::v1::api::run::{
    init_logging, new_run_id, DryRunExecutor, Executor, PathSpec, RunSpec,
};

use crate::commands::cli::{render, Cli};
use crate::commands::validation::{ensure_profile_run_base_dir, load_profile_for_cli};

pub(crate) mod bam;
pub(crate) mod bench;
pub mod cli;
pub(crate) mod fastq;
pub(crate) mod formatting;
pub(crate) mod imports;
pub(crate) mod rendering;
pub(crate) mod validation;

include!("other.rs");
include!("policies.rs");
