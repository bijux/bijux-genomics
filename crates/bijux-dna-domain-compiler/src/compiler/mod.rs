use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_infra::{ensure_dir, write_string};
use serde::{Deserialize, Serialize};

pub mod bundle;
mod compile;
mod coverage;
mod loading;
mod models;
mod support;
mod validation;
mod vcf_emit;

use self::models::{
    AdapterBank, BenchmarkScenario, ContainerCatalog, ContainerToolContract, ContaminationDbBank,
    DomainArtifactVocabulary, DomainIndex, DomainMetricVocabulary, DomainStage, DomainTool,
    DomainToolLoose, ReferenceBank, StageDefaultMap, StageDefaultRationaleMap, StageOutputKindsMap,
    StagePlannedMap, StageResourceHint, StageStatusMap, StageToolMap, ThresholdBand, ToolMap,
    ToolRow,
};
use self::support::{
    collect_yaml_files, default_healthcheck_cmd, default_version_regex, encode_f64_map,
    encode_threshold_map, ensure_no_placeholders_in_active_config, ensure_status, generated_header,
    generation_content_hash, git_head_commit, has_supported_placeholder_forbidden_token,
    infer_tool_role, is_tool_meaningful_in_domain, is_umbrella_stage, is_unspecified,
    normalize_generated_text, placeholders_allowed, read_yaml, required_tool_roles_for_stage,
    resolve_tool_citation, scope_active, toml_array, validate_tool_output_subset,
};

pub use self::compile::compile_domain_configs;
pub use self::coverage::domain_coverage_report;
pub use self::models::{
    CompileOptions, ValidateOptions, DEFAULT_COMPILE_SCOPE, DEFAULT_CONFIGS_DIR, DEFAULT_DOMAIN_DIR,
};
pub use self::validation::validate_domain;
