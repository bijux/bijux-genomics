//! Private admitted identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_GL_PROPAGATION_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.gl_propagation";

pub(crate) const LOCAL_VCF_GL_PROPAGATION_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_gl_propagation_smoke.v1";

pub(crate) const LOCAL_VCF_GL_PROPAGATION_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_gl_propagation_smoke.metrics.v1";

pub(crate) const LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-gl-propagation-smoke";

pub(crate) const GOVERNED_VCF_GL_PROPAGATION_STAGE_ID: &str = "vcf.gl_propagation";

pub(crate) const GOVERNED_VCF_GL_PROPAGATION_TOOL_ID: &str = "bcftools";

pub(crate) const GOVERNED_VCF_GL_PROPAGATION_CORPUS_ID: &str = "vcf_production_regression";

pub(crate) const GOVERNED_VCF_GL_PROPAGATION_ASSET_PROFILE_ID: &str = "vcf_single_sample";

pub(crate) const GOVERNED_VCF_GL_PROPAGATION_INPUT_FIXTURE_ID: &str = "likelihood_single_sample";

pub(crate) const GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME: &str = "sample_a";

pub(crate) const DEFAULT_INPUT_VCF_NAME: &str = "gl_input.vcf";

pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "propagated.vcf.gz";

pub(crate) const DEFAULT_OUTPUT_BCF_NAME: &str = "propagated.bcf";

pub(crate) const DEFAULT_OUTPUT_REPORT_NAME: &str = "gl_propagation_report.json";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

pub(crate) const GOVERNED_RETAINED_LIKELIHOOD_FIELDS: &[&str] = &["GL", "PL", "GP"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfGlPropagationSmokeContract {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) sample_name: String,
}
