use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_infra::{ensure_dir, write_string};
use serde::{Deserialize, Serialize};

include!("compiler_sections/domain_models_and_utils.rs");
include!("compiler_sections/domain_loading_and_collection.rs");
include!("compiler_sections/compile_configs.rs");
include!("compiler_sections/validation.rs");
include!("compiler_sections/coverage_report_and_contracts.rs");
