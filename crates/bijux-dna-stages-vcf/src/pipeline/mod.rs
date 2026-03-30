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
mod imputation;
mod orchestration;
mod population_panel;
mod qc;
include!("../pipeline_sections/execution/chunking_and_resume.rs");

pub(crate) use calling::{
    format_has_token, parse_af_from_info, parse_info_value_f64, parse_record_fields,
    read_vcf_text, variant_key,
};
pub(crate) use imputation::{canonical_contig_label, run_impute_stage_inner};
pub(crate) use orchestration::{
    license_metadata_for_tool_exists, load_imputation_qc_thresholds, parse_format_index,
    resolve_tool_digest, workspace_root,
};
pub(crate) use qc::resolve_phasing_backend;
pub use calling::*;
pub use imputation::*;
pub use orchestration::*;
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
