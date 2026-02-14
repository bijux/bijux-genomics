use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_domain_vcf::contracts::{
    EntryVcfInvariantState, PanelMapInvariantState, PanelSelectionContext, SpeciesContext,
};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use serde::Serialize;
use sha2::Digest;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfPanelLock {
    pub panel_id: String,
    pub reference_build: String,
    pub panel_checksum_sha256: String,
    pub index_checksum_sha256: String,
    pub license_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct VcfPipelineInputs {
    pub policy: PlanPolicy,
    pub coverage_regime: CoverageRegime,
    #[serde(default)]
    pub mean_depth_x: Option<f64>,
    pub vcf: PathBuf,
    pub out_dir: PathBuf,
    #[serde(default)]
    pub stage_tool_overrides: BTreeMap<String, String>,
    #[serde(default)]
    pub requested_stages: Option<Vec<String>>,
    #[serde(default)]
    pub panel_locks: Vec<VcfPanelLock>,
    #[serde(default)]
    pub panel_id: Option<String>,
    #[serde(default)]
    pub map_id: Option<String>,
    pub panel_selection: PanelSelectionContext,
    pub species_context: SpeciesContext,
    pub entry_vcf_invariants: EntryVcfInvariantState,
    pub panel_map_invariants: PanelMapInvariantState,
    pub pipeline_domain: String,
    #[serde(default)]
    pub chunking: ChunkPlanSettings,
    #[serde(default)]
    pub stage_param_overrides: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChunkPlanSettings {
    pub window_size_bp: u64,
    pub overlap_bp: u64,
    pub chr_include: Vec<String>,
    pub chr_exclude: Vec<String>,
    pub max_parallel_chunks: usize,
}

impl Default for ChunkPlanSettings {
    fn default() -> Self {
        Self {
            window_size_bp: 5_000_000,
            overlap_bp: 100_000,
            chr_include: Vec::new(),
            chr_exclude: Vec::new(),
            max_parallel_chunks: 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RegionChunkPlan {
    pub chunk_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PlannerExplainStage {
    pub stage_id: String,
    pub selected_tool: String,
    pub reason: String,
    pub coverage_regime: CoverageRegime,
    pub params_surface: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PlannerExplainV1 {
    pub schema_version: String,
    pub planner_version: String,
    pub coverage_regime: CoverageRegime,
    pub backend_selection_reason: String,
    pub panel_selection_reason: String,
    pub map_selection_reason: String,
    pub chunking_selection_reason: String,
    pub resolved_reference_bundle_id: String,
    pub resolved_reference_lock: String,
    pub resolved_coverage_profile: Option<String>,
    pub resolved_coverage_regime: CoverageRegime,
    pub coverage_resolution_reason: String,
    pub damage_aware_policy: serde_json::Value,
    pub selected_panel: Option<VcfPanelLock>,
    pub decision_traces: Vec<serde_json::Value>,
    pub stages: Vec<PlannerExplainStage>,
}

pub(crate) fn short_species_context_digest(
    species_id: &str,
    build_id: &str,
    contig_set_digest: &str,
) -> String {
    let seed = format!("{species_id}|{build_id}|{contig_set_digest}");
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    let full = format!("{:x}", hasher.finalize());
    full.chars().take(12).collect()
}
