use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_infra::{ensure_dir, write_string};
use serde::{Deserialize, Serialize};

mod compile;
mod coverage;
mod loading;
mod models;
mod validation;

use self::models::*;

include!("shared.rs");

pub use self::compile::compile_domain_configs;
pub use self::coverage::domain_coverage_report;
pub use self::models::{CompileOptions, ValidateOptions};
pub use self::validation::validate_domain;
