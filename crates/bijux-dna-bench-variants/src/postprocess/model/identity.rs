//! Admitted benchmark identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_POSTPROCESS_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.postprocess";

pub(crate) const LOCAL_VCF_POSTPROCESS_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_postprocess_smoke.v1";

pub(crate) const LOCAL_VCF_POSTPROCESS_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_postprocess_smoke.metrics.v1";

pub(crate) const LOCAL_VCF_POSTPROCESS_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-postprocess-smoke";

pub(crate) const GOVERNED_VCF_POSTPROCESS_STAGE_ID: &str = "vcf.postprocess";

pub(crate) const GOVERNED_VCF_POSTPROCESS_TOOL_ID: &str = "bcftools";

pub(crate) const GOVERNED_VCF_POSTPROCESS_CORPUS_ID: &str = "vcf_production_regression";

pub(crate) const GOVERNED_VCF_POSTPROCESS_ASSET_PROFILE_ID: &str = "vcf_single_sample";

pub(crate) const GOVERNED_VCF_POSTPROCESS_INPUT_FIXTURE_ID: &str =
    "multiallelic_normalization_single_sample";

pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "postprocess_input.vcf";

pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "postprocess.vcf.gz";

pub(crate) const DEFAULT_OUTPUT_VALIDATE_NAME: &str = "validate_outputs.json";

pub(crate) const DEFAULT_OUTPUT_MANIFEST_NAME: &str = "final_manifest.json";

pub(crate) const DEFAULT_OUTPUT_NORMALIZATION_NAME: &str = "normalization_contract.json";

pub(crate) const DEFAULT_OUTPUT_CHECKSUMS_NAME: &str = "artifact_checksums.json";

pub(crate) const DEFAULT_OUTPUT_LOGS_NAME: &str = "logs.txt";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfPostprocessSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) input_fixture: String,
}
