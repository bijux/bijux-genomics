//! Owner: bijux-dna-bench-population
//! Persisted population report contracts and admitted stage identities.

pub(crate) const DEFAULT_VCF_IBD_SMOKE_ROOT: &str = "artifacts/runs/bench/local-smoke/vcf.ibd";
pub(crate) const LOCAL_VCF_IBD_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-ibd-smoke";
pub(crate) const GOVERNED_VCF_IBD_STAGE_ID: &str = bijux_dna_core::id_catalog::VCF_IBD;
pub(crate) const GOVERNED_VCF_IBD_CORPUS_ID: &str = "vcf_production_regression";
pub(crate) const GOVERNED_VCF_IBD_ASSET_PROFILE_ID: &str = "vcf_cohort";
pub(crate) const GOVERNED_VCF_IBD_INPUT_FIXTURE_ID: &str = "vcf_mini_multisample_cohort";
pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "ibd_input.vcf";
pub(crate) const DEFAULT_INPUT_SAMPLE_METADATA_NAME: &str = "sample_metadata.tsv";
pub(crate) const DEFAULT_OUTPUT_TSV_NAME: &str = "ibd.tsv";
pub(crate) const DEFAULT_OUTPUT_JSON_NAME: &str = "ibd.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_INPUT_NAME: &str = "source_ibd_input.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME: &str = "source_ibd_segments.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME: &str =
    "source_ibd_merged_segments.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME: &str =
    "source_ibd_filtered_segments.tsv";
pub(crate) const DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME: &str = "source_ibd_summary.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_METRICS_NAME: &str = "source_ibd_metrics.json";
pub(crate) const DEFAULT_OUTPUT_SOURCE_LOGS_NAME: &str = "source_logs.txt";
pub(crate) const DEFAULT_PROBE_INPUT_VCF_NAME: &str = "probe_sparse_overlap.vcf";
pub(crate) const DEFAULT_PROBE_SOURCE_SUMMARY_NAME: &str = "probe_source_ibd_summary.json";
pub(crate) const DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME: &str =
    "probe_source_ibd_filtered_segments.tsv";
pub(crate) const DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME: &str = "probe_source_roh_summary.json";
pub(crate) const DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME: &str = "probe_source_roh_segments.tsv";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";
pub(crate) const GOVERNED_IBD_SEGMENT_MARKER_MINIMUM: usize = 1;
pub(crate) const PROBE_IBD_SEGMENT_MARKER_MINIMUM: usize = 50;
pub(crate) const VCF_BENCHMARK_MUTATOR_LOCK_PATH: &str =
    "artifacts/test-locks/vcf-benchmark-mutators.lock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfIbdSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParsedIbdSegment {
    pub(crate) sample_a: String,
    pub(crate) sample_b: String,
    pub(crate) contig: String,
    pub(crate) start: u64,
    pub(crate) end: u64,
    pub(crate) length_cm: f64,
    pub(crate) marker_count: u64,
}
