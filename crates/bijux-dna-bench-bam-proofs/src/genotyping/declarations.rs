#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::BAM_GENOTYPING;
pub const DEFAULT_BAM_GENOTYPING_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.genotyping.complete.json";

#[cfg(feature = "bam_downstream")]
pub(super) const BAM_GENOTYPING_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_genotyping_complete.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_STAGE_ID: &str = BAM_GENOTYPING;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_TOOL_ID: &str = "bcftools";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_SAMPLE_ID: &str = "human_like_genotyping_candidate_panel";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_REFERENCE_FASTA: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_SITES_VCF: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_REGIONS: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_BAM_PIPELINE_ID: &str = "bam-genotyping";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_CROSS_PIPELINE_ID: &str = "bam-genotyping-to-vcf-downstream";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_VCF_STATS_TOOL_ID: &str = "bcftools";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_VCF_QC_TOOL_ID: &str = "bcftools";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_PARSER_SCHEMA_VERSION: &str = "bijux.bam.genotyping.v1";

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_PARSER_CALL_RATE: f64 = 1.0;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_PARSER_MEAN_POSTERIOR: f64 = 0.99;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_MIN_POSTERIOR: f64 = 0.9;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_MIN_CALL_RATE: f64 = 0.5;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_VARIANT_COUNT: u64 = 4;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_SAMPLE_COUNT: u64 = 2;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_SAMPLE_MISSINGNESS_THRESHOLD: f64 = 0.5;

#[cfg(feature = "bam_downstream")]
pub(super) const EXPECTED_VARIANT_MISSINGNESS_THRESHOLD: f64 = 0.5;

#[cfg(feature = "bam_downstream")]
pub(super) const CHECKED_SURFACE_COUNT: usize = 15;

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_OUTPUT_IDS: [&str; 3] = ["genotyping_report", "summary", "stage_metrics"];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_PLAN_OUTPUT_IDS: [&str; 7] = [
    "genotyping_report",
    "genotyping_bcf",
    "genotyping_vcf",
    "genotyping_vcf_tbi",
    "genotyping_gl",
    "summary",
    "stage_metrics",
];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_BAM_PIPELINE_OUTPUTS: [&str; 6] = [
    "genotyping_bcf",
    "genotyping_vcf_gz",
    "genotyping_vcf_tbi",
    "genotyping_gl_json",
    "genotyping_report_json",
    "genotyping_stage_metrics",
];

#[cfg(feature = "bam_downstream")]
pub(super) const REQUIRED_VCF_FILTER_UPSTREAM_INPUTS: [&str; 3] =
    ["genotyping_vcf_gz", "genotyping_vcf_tbi", "genotyping_report_json"];
