//! Persisted reference panel contracts and declared identities.
use serde::Serialize;
use std::collections::BTreeMap;

pub(crate) const DEFAULT_VCF_PREPARE_PANEL_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.prepare_reference_panel";
pub(crate) const LOCAL_VCF_PREPARE_PANEL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_prepare_reference_panel_smoke.v1";
pub(crate) const LOCAL_VCF_PREPARE_PANEL_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_prepare_reference_panel_smoke.metrics.v1";
pub(crate) const LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-prepare-reference-panel-smoke";
pub(crate) const GOVERNED_VCF_PREPARE_PANEL_STAGE_ID: &str = "vcf.prepare_reference_panel";
pub(crate) const GOVERNED_VCF_PREPARE_PANEL_TOOL_ID: &str = "bcftools";
pub(crate) const GOVERNED_VCF_PREPARE_PANEL_CORPUS_ID: &str = "vcf_production_regression";
pub(crate) const GOVERNED_VCF_PREPARE_PANEL_ASSET_PROFILE_ID: &str = "vcf_reference_panel";
pub(crate) const GOVERNED_VCF_PREPARE_PANEL_PANEL_ID: &str = "hsapiens_grch38_1kg_3202_chr22";
pub(crate) const GOVERNED_VCF_PREPARE_PANEL_MAP_ID: &str = "hsapiens_grch38_glimpse_chr22";
pub(crate) const GOVERNED_VCF_PREPARE_PANEL_INPUT_FIXTURE_ID: &str =
    "reference_panel_sort_and_deduplicate";
pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "prepare_reference_panel_input.vcf";
pub(crate) const DEFAULT_RAW_PANEL_NAME: &str = "panel.vcf.gz";
pub(crate) const DEFAULT_OUTPUT_PANEL_NAME: &str = "panel.vcf.gz";
pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
pub(crate) const DEFAULT_PANEL_MANIFEST_NAME: &str = "panel_manifest.json";
pub(crate) const DEFAULT_OVERLAP_NAME: &str = "overlap.json";
pub(crate) const DEFAULT_PANEL_OVERLAP_NAME: &str = "panel_overlap.json";
pub(crate) const DEFAULT_PANEL_FILES_NAME: &str = "panel_files.json";
pub(crate) const DEFAULT_OVERLAP_TSV_NAME: &str = "overlap.tsv";
pub(crate) const DEFAULT_CHUNKS_NAME: &str = "chunks.json";
pub(crate) const EXPECTED_SAMPLE_ID: &str = "sample1";
pub(crate) const EXPECTED_INPUT_VARIANT_COUNT: u64 = 5;
pub(crate) const EXPECTED_OUTPUT_VARIANT_COUNT: u64 = 4;
pub(crate) const EXPECTED_DUPLICATE_SITES_REMOVED: u64 = 1;
pub(crate) const EXPECTED_NORMALIZATION_STATUS: &str = "sorted_indexed_deduplicated";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfPrepareReferencePanelSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
    pub(crate) panel: String,
    pub(crate) map: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVcfPrepareReferencePanelSmokeMetrics {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub input_variants: u64,
    pub output_variants: u64,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub duplicate_sites_removed: u64,
    pub normalization_status: String,
    pub index_path: String,
    pub panel_id: String,
    pub map_id: String,
    pub tool_id: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
// Independent validation flags are preserved by the persisted report schema.
#[allow(clippy::struct_excessive_bools)]
pub struct LocalVcfPrepareReferencePanelSmokeReport {
    pub schema_version: &'static str,
    pub command: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub input_fixture_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub input_vcf_path: String,
    pub raw_panel_path: String,
    pub output_root: String,
    pub panel_vcf_path: String,
    pub panel_tbi_path: String,
    pub panel_manifest_path: String,
    pub overlap_path: String,
    pub panel_overlap_path: String,
    pub panel_files_path: String,
    pub overlap_tsv_path: String,
    pub chunks_path: String,
    pub metrics_path: String,
    pub stage_result_manifest_path: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub input_variants: u64,
    pub output_variants: u64,
    pub sample_count: u64,
    pub sample_ids: Vec<String>,
    pub sample_consistent: bool,
    pub duplicate_sites_removed: u64,
    pub normalization_status: String,
    pub index_path: String,
    pub parseable: bool,
    pub validation_checks: BTreeMap<String, bool>,
    pub gt_present: bool,
    pub gl_present: bool,
}
