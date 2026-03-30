use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use bijux_dna_db_ref::{
    ref_service, resolve_map_lock, resolve_panel_lock, validate_imputation_tool_compatibility,
};
use bijux_dna_domain_vcf::{
    contracts::SpeciesContext,
    params::{VcfCallParams, VcfFilterParams, VcfStatsParams},
    VcfStatsMetricsV1,
};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::metrics::{parse_depth_from_info, parse_vcf_call_summary, parse_vcf_filter_breakdown};

// Runtime digest contract markers:
// "tool_digest": resolve_tool_digest
// "tool_digest": tool_digest

mod calling;
mod population_panel;
mod qc;
include!("../pipeline_sections/execution/runtime_and_orchestration.rs");
include!("../pipeline_sections/imputation/impute_and_postprocess.rs");
include!("../pipeline_sections/execution/chunking_and_resume.rs");

pub use calling::*;
pub use population_panel::*;
pub use qc::*;

fn resolve_panel(
    species: &str,
    build: &str,
    panel_id: Option<&str>,
) -> Result<bijux_dna_db_ref::PanelCatalogEntry> {
    ref_service().resolve_panel(species, build, panel_id)
}

fn resolve_map(
    species: &str,
    build: &str,
    map_id: Option<&str>,
) -> Result<bijux_dna_db_ref::MapCatalogEntry> {
    ref_service().resolve_map(species, build, map_id)
}
