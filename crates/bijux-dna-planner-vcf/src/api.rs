use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_domain_vcf::contracts::{
    EntryVcfInvariantState, PanelMapInvariantState, PanelSelectionContext, SpeciesContext,
};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use serde::Serialize;

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
    #[serde(default)]
    pub call_bam: Option<PathBuf>,
    #[serde(default)]
    pub call_bam_index: Option<PathBuf>,
    #[serde(default)]
    pub reference_fasta: Option<PathBuf>,
    #[serde(default)]
    pub reference_panel_vcf: Option<PathBuf>,
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
