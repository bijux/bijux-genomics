//! Private admitted identity and governed artifact declarations.

pub(crate) const DEFAULT_VCF_CALL_PSEUDOHAPLOID_SMOKE_ROOT: &str =
    "artifacts/runs/bench/local-smoke/vcf.call_pseudohaploid";

pub(crate) const LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_pseudohaploid_smoke.v1";

pub(crate) const LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_call_pseudohaploid_smoke.metrics.v1";

pub(crate) const LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_COMMAND: &str =
    "bijux-dna bench local run-vcf-call-pseudohaploid-smoke";

pub(crate) const GOVERNED_VCF_CALL_PSEUDOHAPLOID_STAGE_ID: &str = "vcf.call_pseudohaploid";

pub(crate) const DEFAULT_OUTPUT_VCF_NAME: &str = "pseudohaploid.vcf.gz";

pub(crate) const DEFAULT_OUTPUT_METRICS_NAME: &str = "metrics.json";

pub(crate) const GOVERNED_PSEUDOHAPLOID_SAMPLING_POLICY: &str =
    "deterministic_first_allele_projection";

pub(crate) const GOVERNED_PSEUDOHAPLOID_SEED_USAGE: &str = "control_seed_for_replay_proof";

pub(crate) const GOVERNED_PSEUDOHAPLOID_RANDOM_SEED: u64 = 73;

pub(crate) const GOVERNED_PSEUDOHAPLOID_DETERMINISM_SCOPE: &str =
    "vcf_payload_without_bcftools_command_headers";
