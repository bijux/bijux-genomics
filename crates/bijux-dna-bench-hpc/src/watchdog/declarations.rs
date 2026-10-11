pub const DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/dag-sim/no-global-wait.json";
pub const DEFAULT_FAILURE_ISOLATION_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/dag-sim/failure-isolation.json";
pub const DEFAULT_PARTIAL_RESUME_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/dag-sim/partial-resume.json";
pub const DEFAULT_COMPLETION_RULES_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/dag-sim/completion-rules.json";
pub(super) const LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_dag_watchdog_simulation.v1";
pub(super) const NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID: &str =
    bijux_dna_core::id_catalog::FASTQ_PROFILE_READ_LENGTHS;
pub(super) const FAILURE_ISOLATION_FAILED_SAMPLE_ID: &str = "sample_alpha";
pub(super) const FAILURE_ISOLATION_CONTINUED_SAMPLE_ID: &str = "sample_beta";
pub(super) const FAILURE_ISOLATION_FAILED_STAGE_ID: &str =
    bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS;
pub(super) const PARTIAL_RESUME_INVALID_NODE_ID: &str = bijux_dna_core::id_catalog::FASTQ_TRIM;
pub(super) const PARTIAL_RESUME_MISSING_NODE_IDS: &[&str] = &[
    bijux_dna_core::id_catalog::FASTQ_FILTER,
    bijux_dna_core::id_catalog::FASTQ_STATS_NEUTRAL,
    bijux_dna_core::id_catalog::FASTQ_QC_POST,
];
pub(super) const COMPLETION_RULES_STAGE_ID: &str = bijux_dna_core::id_catalog::FASTQ_FILTER;
