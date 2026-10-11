//! Admitted benchmark identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_DAMAGE_FILTER_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.damage_filter";

pub(crate) const LOCAL_VCF_DAMAGE_FILTER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_damage_filter_smoke.v1";

pub(crate) const LOCAL_VCF_DAMAGE_FILTER_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_damage_filter_smoke.metrics.v1";

pub(crate) const LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-damage-filter-smoke";

pub(crate) const GOVERNED_VCF_DAMAGE_FILTER_STAGE_ID: &str = "vcf.damage_filter";

pub(crate) const GOVERNED_VCF_DAMAGE_FILTER_TOOL_ID: &str = "bcftools";

pub(crate) const GOVERNED_VCF_DAMAGE_FILTER_CORPUS_ID: &str = "vcf_production_regression";

pub(crate) const GOVERNED_VCF_DAMAGE_FILTER_ASSET_PROFILE_ID: &str = "vcf_single_sample";

pub(crate) const GOVERNED_VCF_DAMAGE_FILTER_INPUT_FIXTURE_ID: &str =
    "terminal_damage_single_sample";

pub(crate) const GOVERNED_VCF_DAMAGE_FILTER_SAMPLE_NAME: &str = "sample_a";

pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "damage_input.vcf";

pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "damage_filtered.vcf.gz";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

pub(crate) const DAMAGE_FILTER_SUMMARY_NAME: &str = "damage_filter_summary.json";

pub(crate) const DAMAGE_FILTER_COUNTS_NAME: &str = "damage_filter_counts.json";

pub(crate) const DAMAGE_FILTER_WARNINGS_NAME: &str = "warnings.json";

pub(crate) const DAMAGE_FILTER_MANIFEST_NAME: &str = "damage_genotype_manifest.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfDamageFilterSmokeContract {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_name: String,
}
