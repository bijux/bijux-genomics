//! Private admitted identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_CALL_DIPLOID_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.call_diploid";

pub(crate) const LOCAL_VCF_CALL_DIPLOID_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_diploid_smoke.v1";

pub(crate) const LOCAL_VCF_CALL_DIPLOID_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_diploid_smoke.metrics.v1";

pub(crate) const LOCAL_VCF_CALL_DIPLOID_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-call-diploid-smoke";

pub(crate) const GOVERNED_VCF_CALL_DIPLOID_STAGE_ID: &str = "vcf.call_diploid";

pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "diploid.vcf.gz";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";
