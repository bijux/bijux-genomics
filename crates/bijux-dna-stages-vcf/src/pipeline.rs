use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use bijux_dna_db_ref::{resolve_map, resolve_panel, validate_imputation_tool_compatibility};
use bijux_dna_domain_vcf::{
    contracts::SpeciesContext,
    params::{VcfCallParams, VcfFilterParams, VcfStatsParams},
    taxonomy::VcfDomainStage,
    VcfStatsMetricsV1,
};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::metrics::{parse_depth_from_info, parse_vcf_call_summary, parse_vcf_filter_breakdown};

// Runtime digest contract markers:
// "tool_digest": resolve_tool_digest
// "tool_digest": tool_digest

include!("pipeline_sections/execution/call_filter_and_gl.rs");
include!("pipeline_sections/qc/qc_and_stage_params.rs");
include!("pipeline_sections/execution/population_and_panel_prep.rs");
include!("pipeline_sections/execution/runtime_and_orchestration.rs");
include!("pipeline_sections/imputation/impute_and_postprocess.rs");
include!("pipeline_sections/execution/chunking_and_resume.rs");
