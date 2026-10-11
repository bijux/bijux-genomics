//! Private admitted identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_CALL_SMOKE_ROOT: &str = "artifacts/runs/bench/local-smoke/vcf.call";

pub(crate) const LOCAL_VCF_CALL_SMOKE_COMMAND: &str = "bijux-dna bench local run-vcf-call-smoke";

pub(crate) const GOVERNED_VCF_CALL_STAGE_ID: &str = "vcf.call";

pub(crate) const GOVERNED_VCF_CALL_RESOLVED_STAGE_ID: &str = "vcf.call_diploid";

pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "calls.vcf.gz";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

pub(crate) const VCF_CALL_SMOKE_LOCK_PATH: &str =
    "artifacts/bench-local-vcf-call-smoke/publish.lock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfCallSmokeContract {
    pub(crate) stage_id: String,
    pub(crate) resolved_stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_bam: String,
    pub(crate) input_bam_index: String,
    pub(crate) reference: String,
    pub(crate) sample_id: String,
    pub(crate) sample_name: String,
}
