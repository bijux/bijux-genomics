//! Admitted benchmark identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_FILTER_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.filter";

pub(crate) const LOCAL_VCF_FILTER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_filter_smoke.v1";

pub(crate) const LOCAL_VCF_FILTER_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_filter_smoke.metrics.v1";

pub(crate) const LOCAL_VCF_FILTER_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-filter-smoke";

pub(crate) const GOVERNED_VCF_FILTER_STAGE_ID: &str = "vcf.filter";

pub(crate) const GOVERNED_VCF_FILTER_TOOL_ID: &str = "bcftools";

pub(crate) const GOVERNED_VCF_FILTER_CORPUS_ID: &str = "vcf_production_regression";

pub(crate) const GOVERNED_VCF_FILTER_ASSET_PROFILE_ID: &str = "vcf_single_sample";

pub(crate) const GOVERNED_VCF_FILTER_INPUT_FIXTURE_ID: &str = "site_filter_single_sample";

pub(crate) const GOVERNED_VCF_FILTER_SAMPLE_NAME: &str = "sample_a";

pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "filter_input.vcf";

pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "filtered.vcf.gz";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

pub(crate) const FILTER_BREAKDOWN_JSON_NAME: &str = "filter_breakdown.json";

pub(crate) const FILTER_BREAKDOWN_TSV_NAME: &str = "filter_breakdown.tsv";

pub(crate) const FILTER_EXPLAIN_JSON_NAME: &str = "filter_explain.json";

pub(crate) const EXPECTED_FILTER_IDS: &[&str] = &["HIGH_MISSING", "LOWQUAL", "LOW_DP", "LOW_MQ"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfFilterSmokeContract {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_name: String,
}
