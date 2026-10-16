use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_api::v1::api::bench::{
    index_canonical_benchmark_result_bindings, reconcile_canonical_benchmark_result_identities,
    CanonicalBenchmarkResultBinding as CanonicalResultIdRow, CanonicalResultIdentityViolation,
};
pub use bijux_dna_api::v1::api::write_metric_registry_report;
use bijux_dna_api::v1::api::{
    read_workspace_crate_manifests as workspace_members, WorkspaceCrateManifest as WorkspaceMember,
};
pub use bijux_dna_api::v1::api::{
    write_workspace_dependency_map as write_dependency_map, CrateDependencyEdge,
};
pub type MetricRegistryReport = bijux_dna_api::v1::api::MetricRegistryReport;
pub type MetricRegistryRow = bijux_dna_api::v1::api::MetricRegistryRow;
use serde::Serialize;
use toml::Value;

mod proof_transport;
use proof_transport::{run_gate_proof, ProofId};

const DOMAIN_CRATES: &[&str] =
    &["bijux-dna-domain-bam", "bijux-dna-domain-fastq", "bijux-dna-domain-vcf"];
const PLANNER_CRATES: &[&str] =
    &["bijux-dna-planner-bam", "bijux-dna-planner-fastq", "bijux-dna-planner-vcf"];
const RUNNER_OWNED_PROCESS_EXECUTION_REPORT_CRATES: &[&str] = &[
    "bijux-dna-analyze",
    "bijux-dna-bench-model",
    "bijux-dna-domain-bam",
    "bijux-dna-domain-compiler",
    "bijux-dna-domain-fastq",
    "bijux-dna-domain-vcf",
    "bijux-dna-stages-bam",
    "bijux-dna-stages-fastq",
    "bijux-dna-stages-vcf",
];

const PROCESS_EXECUTION_PATTERNS: &[&str] = &[
    concat!("Command", "::new"),
    concat!("std::process::", "Command"),
    concat!("tokio::process::", "Command"),
];
const CONTAINER_EXECUTION_PATTERNS: &[&str] = &[
    concat!("Command", "::new(\"docker\")"),
    concat!("Command", "::new(\"apptainer\")"),
    concat!("Command", "::new(\"singularity\")"),
    concat!("Command", "::new(\"podman\")"),
    concat!("std::process::", "Command", "::new(\"docker\")"),
    concat!("std::process::", "Command", "::new(\"apptainer\")"),
    concat!("std::process::", "Command", "::new(\"singularity\")"),
    concat!("std::process::", "Command", "::new(\"podman\")"),
    concat!("tokio::process::", "Command", "::new(\"docker\")"),
    concat!("tokio::process::", "Command", "::new(\"apptainer\")"),
    concat!("tokio::process::", "Command", "::new(\"singularity\")"),
    concat!("tokio::process::", "Command", "::new(\"podman\")"),
];
const SLURM_EXECUTION_PATTERNS: &[&str] = &[
    concat!("Command", "::new(\"sbatch\")"),
    concat!("Command", "::new(\"srun\")"),
    concat!("Command", "::new(\"salloc\")"),
    concat!("std::process::", "Command", "::new(\"sbatch\")"),
    concat!("std::process::", "Command", "::new(\"srun\")"),
    concat!("std::process::", "Command", "::new(\"salloc\")"),
    concat!("tokio::process::", "Command", "::new(\"sbatch\")"),
    concat!("tokio::process::", "Command", "::new(\"srun\")"),
    concat!("tokio::process::", "Command", "::new(\"salloc\")"),
];

const RUNNER_DEPENDENCY_PATTERNS: &[&str] = &["runner"];
const CONTAINER_DEPENDENCY_PATTERNS: &[&str] =
    &["docker", "apptainer", "singularity", "podman", "container"];
const SLURM_DEPENDENCY_PATTERNS: &[&str] = &["slurm"];
const PARSER_ENTRYPOINT_PATTERNS: &[&str] = &["pub fn parse_", "pub(crate) fn parse_"];
const RAW_INPUT_READ_PATTERNS: &[&str] = &[
    "fs::read_to_string",
    "std::fs::read_to_string",
    "fs::read(",
    "std::fs::read(",
    "File::open(",
    ".read_to_string(",
];
const INPUT_MUTATION_PATTERNS: &[&str] = &[
    concat!("fs", "::write("),
    concat!("std::fs", "::write("),
    concat!("File", "::create("),
    concat!("fs", "::create_dir("),
    concat!("fs", "::create_dir_all("),
    concat!("std::fs", "::create_dir("),
    concat!("std::fs", "::create_dir_all("),
    concat!("fs", "::remove_file("),
    concat!("fs", "::remove_dir("),
    concat!("fs", "::remove_dir_all("),
    concat!("std::fs", "::remove_file("),
    concat!("std::fs", "::remove_dir("),
    concat!("std::fs", "::remove_dir_all("),
    concat!("fs", "::rename("),
    concat!("std::fs", "::rename("),
    concat!("fs", "::copy("),
    concat!("std::fs", "::copy("),
    "atomic_write",
    "write_bytes(",
    "write_json(",
];
const PLANNER_PARSER_API_PATTERNS: &[&str] = &[
    "bijux_dna_domain_fastq::observer",
    "bijux_dna_stages_fastq::observer",
    "observer::parse_",
    "metrics::parse_",
    "parsers::parse_",
    "bijux_dna_domain_vcf::parsers",
    "raw_parser_contract",
];

#[derive(Clone, Copy, Debug)]
struct ParserSurfaceSpec {
    crate_name: &'static str,
    parser_root: &'static str,
    excluded_paths: &'static [ParserSurfaceExclusionSpec],
}

#[derive(Clone, Copy, Debug)]
struct ParserSurfaceExclusionSpec {
    path: &'static str,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct PlannerCrateAuditSpec {
    crate_name: &'static str,
    allowed_input_read_files: &'static [AllowedAuditFileSpec],
}

#[derive(Clone, Copy, Debug)]
struct AllowedAuditFileSpec {
    path: &'static str,
    reason: &'static str,
}

const PARSER_SURFACES: &[ParserSurfaceSpec] = &[
    ParserSurfaceSpec {
        crate_name: "bijux-dna-domain-bam",
        parser_root: "src/metrics",
        excluded_paths: &[ParserSurfaceExclusionSpec {
            path: "src/metrics/raw_parser_contract.rs",
            reason: "governance helper for malformed raw-fixture probe generation",
        }],
    },
    ParserSurfaceSpec {
        crate_name: "bijux-dna-domain-fastq",
        parser_root: "src/observer/parse",
        excluded_paths: &[
            ParserSurfaceExclusionSpec {
                path: "src/observer/parse/parser_contracts",
                reason: "cfg(test) parser contract fixture-bank suite",
            },
            ParserSurfaceExclusionSpec {
                path: "src/observer/parse/raw_parser_contract.rs",
                reason: "governance helper for malformed raw-fixture probe generation",
            },
        ],
    },
    ParserSurfaceSpec {
        crate_name: "bijux-dna-domain-vcf",
        parser_root: "src/parsers",
        excluded_paths: &[],
    },
];

const PLANNER_AUDIT_SPECS: &[PlannerCrateAuditSpec] = &[
    PlannerCrateAuditSpec {
        crate_name: "bijux-dna-planner-bam",
        allowed_input_read_files: &[
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/local_readiness.rs",
                reason: "loads governed local-ready planning inputs and runtime defaults",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/ancient/authenticity.rs",
                reason: "loads governed BAM authenticity case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/ancient/damage.rs",
                reason: "loads governed BAM damage case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/ancient/sex.rs",
                reason: "loads governed BAM sex case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/downstream/bias_mitigation.rs",
                reason: "loads governed BAM bias mitigation case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/downstream/kinship.rs",
                reason: "loads governed BAM kinship case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/filter.rs",
                reason: "loads governed BAM filter case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/length_filter.rs",
                reason: "loads governed BAM length filter case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/mapping_summary.rs",
                reason: "loads governed BAM mapping summary case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/mapq_filter.rs",
                reason: "loads governed BAM mapq filter case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/overlap_correction.rs",
                reason: "loads governed BAM overlap correction case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/qc_pre.rs",
                reason: "loads governed BAM qc pre case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/validate.rs",
                reason: "loads governed BAM validate case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/complexity.rs",
                reason: "loads governed BAM complexity case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/coverage.rs",
                reason: "loads governed BAM coverage case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/duplication_metrics.rs",
                reason: "loads governed BAM duplication metrics case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/endogenous_content.rs",
                reason: "loads governed BAM endogenous content case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/gc_bias.rs",
                reason: "loads governed BAM gc bias case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/insert_size.rs",
                reason: "loads governed BAM insert size case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/markdup.rs",
                reason: "loads governed BAM markdup case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/recalibration.rs",
                reason: "loads governed BAM recalibration case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/selection/domain_tool_specs.rs",
                reason: "loads governed BAM tool planning metadata",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/selection/domain_tool_output_contracts.rs",
                reason: "loads governed BAM output-contract metadata",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/selection/registry.rs",
                reason: "reads the governed workspace tool registry snapshot",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/stage_activation.rs",
                reason: "reads governed stage activation policy",
            },
        ],
    },
    PlannerCrateAuditSpec {
        crate_name: "bijux-dna-planner-fastq",
        allowed_input_read_files: &[
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/local_readiness.rs",
                reason: "loads governed local-ready FASTQ planning inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/cluster_otus.rs",
                reason: "loads governed FASTQ cluster otus case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/infer_asvs.rs",
                reason: "loads governed FASTQ infer asvs case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/normalize_abundance.rs",
                reason: "loads governed FASTQ normalize abundance case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/normalize_primers.rs",
                reason: "loads governed FASTQ normalize primers case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/remove_chimeras.rs",
                reason: "loads governed FASTQ remove chimeras case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/detect_adapters.rs",
                reason: "loads governed FASTQ detect adapters case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/detect_duplicates_premerge.rs",
                reason: "loads governed FASTQ detect duplicates premerge case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/estimate_library_complexity_prealign.rs",
                reason: "loads governed FASTQ estimate library complexity prealign case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/profile_overrepresented_sequences.rs",
                reason: "loads governed FASTQ profile overrepresented sequences case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/profile_read_lengths.rs",
                reason: "loads governed FASTQ profile read lengths case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/validate_reads.rs",
                reason: "loads governed FASTQ validate reads case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/qc/profile_reads.rs",
                reason: "loads governed FASTQ profile reads case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/qc/report_qc.rs",
                reason: "loads governed FASTQ report qc case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/correct_errors.rs",
                reason: "loads governed FASTQ correct errors case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/extract_umis.rs",
                reason: "loads governed FASTQ extract umis case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/filter_low_complexity.rs",
                reason: "loads governed FASTQ filter low complexity case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/filter_reads.rs",
                reason: "loads governed FASTQ filter reads case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/merge_pairs.rs",
                reason: "loads governed FASTQ merge pairs case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/remove_duplicates.rs",
                reason: "loads governed FASTQ remove duplicates case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/trim_polyg_tails.rs",
                reason: "loads governed FASTQ trim polyg tails case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/trim_reads.rs",
                reason: "loads governed FASTQ trim reads case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/trim_terminal_damage.rs",
                reason: "loads governed FASTQ trim terminal damage case inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/quality_sampling.rs",
                reason: "samples governed input reads for planning-time quality estimation",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/selection/domain_tool_specs.rs",
                reason: "loads governed FASTQ tool planning metadata",
            },
            AllowedAuditFileSpec {
                path:
                    "crates/bijux-dna-planner-fastq/src/selection/domain_tool_output_contracts.rs",
                reason: "loads governed FASTQ output-contract metadata",
            },
        ],
    },
    PlannerCrateAuditSpec {
        crate_name: "bijux-dna-planner-vcf",
        allowed_input_read_files: &[
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-vcf/src/coverage.rs",
                reason: "reads governed coverage-regime planning thresholds",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-vcf/src/workspace_config.rs",
                reason: "reads governed workspace tool and parameter registries",
            },
        ],
    },
];

#[derive(Clone, Copy, Debug)]
struct CrateCycleCategorySpec {
    category: &'static str,
    crates: &'static [&'static str],
}

const CRATE_CYCLE_DOMAIN_CRATES: &[&str] = &[
    "bijux-dna-domain-bam",
    "bijux-dna-domain-compiler",
    "bijux-dna-domain-fastq",
    "bijux-dna-domain-vcf",
];
const CRATE_CYCLE_PLANNER_CRATES: &[&str] =
    &["bijux-dna-planner-bam", "bijux-dna-planner-fastq", "bijux-dna-planner-vcf"];
const CRATE_CYCLE_RUNNER_CRATES: &[&str] =
    &["bijux-dna-engine", "bijux-dna-runner", "bijux-dna-runtime"];
const CRATE_CYCLE_PARSER_CRATES: &[&str] = &[];
const CRATE_CYCLE_SCIENCE_CRATES: &[&str] = &["bijux-dna-analyze", "bijux-dna-science"];
const CRATE_CYCLE_BENCHMARK_CRATES: &[&str] = &["bijux-dna-bench", "bijux-dna-bench-model"];
const CRATE_CYCLE_CLI_CRATES: &[&str] = &["bijux-dna", "bijux-dna-dev"];

const CRATE_CYCLE_CATEGORIES: &[CrateCycleCategorySpec] = &[
    CrateCycleCategorySpec { category: "domain", crates: CRATE_CYCLE_DOMAIN_CRATES },
    CrateCycleCategorySpec { category: "planner", crates: CRATE_CYCLE_PLANNER_CRATES },
    CrateCycleCategorySpec { category: "runner", crates: CRATE_CYCLE_RUNNER_CRATES },
    CrateCycleCategorySpec { category: "parser", crates: CRATE_CYCLE_PARSER_CRATES },
    CrateCycleCategorySpec { category: "science", crates: CRATE_CYCLE_SCIENCE_CRATES },
    CrateCycleCategorySpec { category: "benchmark", crates: CRATE_CYCLE_BENCHMARK_CRATES },
    CrateCycleCategorySpec { category: "cli", crates: CRATE_CYCLE_CLI_CRATES },
];

const DEFAULT_CRATE_DEPENDENCY_MAP_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/crate-dependency-map.json";
const DEFAULT_DOMAIN_NO_EXECUTION_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/domain-no-execution.json";
const DEFAULT_PARSER_NO_EXECUTION_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/parser-no-execution.json";
const DEFAULT_PLANNER_NO_PARSER_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/planner-no-parser.json";
const DEFAULT_RUNNER_OWNS_PROCESS_EXECUTION_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/runner-owns-process-execution.json";
const DEFAULT_METRIC_REGISTRY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/metric-registry.tsv";
const DEFAULT_RESULT_ID_STABILITY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/result-id-stability.json";
const DEFAULT_NO_CRATE_CYCLES_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/no-crate-cycles.json";
const DEFAULT_CRATE_SHAPE_GATE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/crates/CRATE_SHAPE_FOR_BENCHMARKING_READY.json";
const DEFAULT_CRATE_SHAPE_GATE_TARGET_DIR: &str = "artifacts/rust/crate-shape-gate-target";
const TYPED_ARTIFACT_HANDOFF_TEST_COMMAND: &str =
    "cargo test -p bijux-dna-api typed_artifact_handoff_rejects_wrong_stage_inputs -- --nocapture";
const METRIC_REGISTRY_PROOF_TEST_COMMAND: &str =
    "cargo test -p bijux-dna-domain-vcf --test contracts vcf_metric_registry_rejects_unregistered_stage_metrics -- --nocapture";

#[derive(Debug, Serialize)]
pub struct DomainNoExecutionReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_crate_count: usize,
    pub ok: bool,
    pub crates: Vec<DomainNoExecutionCrateReport>,
}

#[derive(Debug, Serialize)]
pub struct DomainNoExecutionCrateReport {
    pub crate_name: String,
    pub manifest_path: String,
    pub scanned_rust_files: Vec<String>,
    pub forbidden_direct_dependencies: Vec<ForbiddenDependencyHit>,
    pub process_execution_refs: Vec<SourcePatternHit>,
    pub container_execution_refs: Vec<SourcePatternHit>,
    pub slurm_execution_refs: Vec<SourcePatternHit>,
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct ParserNoExecutionReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_surface_count: usize,
    pub ok: bool,
    pub surfaces: Vec<ParserSurfaceAuditReport>,
}

#[derive(Debug, Serialize)]
pub struct PlannerNoParserReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_crate_count: usize,
    pub ok: bool,
    pub crates: Vec<PlannerNoParserCrateReport>,
}

#[derive(Debug, Serialize)]
pub struct NoCrateCyclesReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_crate_count: usize,
    pub skipped_workspace_crate_count: usize,
    pub category_count: usize,
    pub edge_count: usize,
    pub cycle_count: usize,
    pub ok: bool,
    pub categories: Vec<NoCrateCyclesCategoryReport>,
    pub crates: Vec<NoCrateCyclesCrateReport>,
    pub edges: Vec<CrateDependencyEdge>,
    pub cycles: Vec<NoCrateCyclesComponent>,
    pub topological_order: Vec<String>,
    pub skipped_workspace_crates: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct NoCrateCyclesCategoryReport {
    pub category: String,
    pub crate_count: usize,
    pub crates: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct NoCrateCyclesCrateReport {
    pub crate_name: String,
    pub category: String,
    pub manifest_path: String,
    pub direct_workspace_dependencies: Vec<String>,
    pub direct_workspace_dependents: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct NoCrateCyclesComponent {
    pub crates: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CrateShapeGateCheckReport {
    pub goal_id: String,
    pub ok: bool,
    pub command: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkingReadyCrateShapeGateReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub cargo_target_dir: String,
    pub ok: bool,
    pub checks: Vec<CrateShapeGateCheckReport>,
    pub passed_goal_ids: Vec<String>,
    pub failed_goal_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RunnerOwnedProcessExecutionReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_crate_count: usize,
    pub ok: bool,
    pub crates: Vec<RunnerOwnedProcessExecutionCrateReport>,
}

#[derive(Debug, Serialize)]
pub struct PlannerNoParserCrateReport {
    pub crate_name: String,
    pub manifest_path: String,
    pub scanned_rust_files: Vec<String>,
    pub allowed_input_read_paths: Vec<AllowedAuditPath>,
    pub planner_input_read_refs: Vec<SourcePatternHit>,
    pub unexpected_input_read_refs: Vec<SourcePatternHit>,
    pub forbidden_parser_api_refs: Vec<SourcePatternHit>,
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct RunnerOwnedProcessExecutionCrateReport {
    pub crate_name: String,
    pub category: String,
    pub manifest_path: String,
    pub scanned_rust_files: Vec<String>,
    pub execution_owner_dependencies: Vec<ExecutionOwnerDependencyHit>,
    pub process_execution_refs: Vec<SourcePatternHit>,
    pub container_execution_refs: Vec<SourcePatternHit>,
    pub slurm_execution_refs: Vec<SourcePatternHit>,
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct ParserSurfaceAuditReport {
    pub crate_name: String,
    pub manifest_path: String,
    pub parser_root: String,
    pub scanned_rust_files: Vec<String>,
    pub excluded_governance_paths: Vec<ExcludedAuditPath>,
    pub parser_entrypoints: Vec<SourcePatternHit>,
    pub raw_input_read_refs: Vec<SourcePatternHit>,
    pub input_mutation_refs: Vec<SourcePatternHit>,
    pub forbidden_direct_dependencies: Vec<ForbiddenDependencyHit>,
    pub process_execution_refs: Vec<SourcePatternHit>,
    pub container_execution_refs: Vec<SourcePatternHit>,
    pub slurm_execution_refs: Vec<SourcePatternHit>,
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct ResultIdStabilityReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub row_count: usize,
    pub local_row_count: usize,
    pub fake_row_count: usize,
    pub report_row_count: usize,
    pub slurm_row_count: usize,
    pub micro_checked_row_count: usize,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<ResultIdStabilityRow>,
    pub violations: Vec<ResultIdStabilityViolation>,
}

#[derive(Debug, Serialize)]
pub struct ResultIdStabilityRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub scope_kind: String,
    pub scope_id: String,
    pub local_result_id: Option<String>,
    pub fake_result_id: Option<String>,
    pub micro_result_id: Option<String>,
    pub report_result_id: Option<String>,
    pub slurm_result_id: Option<String>,
    pub local_execution_argv: Option<Vec<String>>,
    pub micro_execution_argv: Option<Vec<String>>,
    pub fake_metrics_path: Option<String>,
    pub report_row_id: Option<String>,
    pub report_evidence_path: Option<String>,
    pub slurm_job_id_local: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResultIdStabilityViolation {
    pub result_id: String,
    pub surface: String,
    pub detail: String,
}

impl From<CanonicalResultIdentityViolation> for ResultIdStabilityViolation {
    fn from(value: CanonicalResultIdentityViolation) -> Self {
        Self { result_id: value.result_id, surface: value.surface, detail: value.detail }
    }
}

#[derive(Debug, Serialize)]
pub struct ForbiddenDependencyHit {
    pub section: String,
    pub dependency: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct SourcePatternHit {
    pub path: String,
    pub line: usize,
    pub pattern: String,
}

type ParserSurfaceHits = (
    Vec<SourcePatternHit>,
    Vec<SourcePatternHit>,
    Vec<SourcePatternHit>,
    Vec<SourcePatternHit>,
    Vec<SourcePatternHit>,
    Vec<SourcePatternHit>,
);

#[derive(Debug, Serialize)]
pub struct ExcludedAuditPath {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct AllowedAuditPath {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionOwnerDependencyHit {
    pub section: String,
    pub dependency: String,
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn crate_cycle_category(crate_name: &str) -> Option<&'static str> {
    CRATE_CYCLE_CATEGORIES
        .iter()
        .find(|spec| spec.crates.contains(&crate_name))
        .map(|spec| spec.category)
}

fn audited_crate_cycle_names() -> BTreeSet<&'static str> {
    CRATE_CYCLE_CATEGORIES.iter().flat_map(|spec| spec.crates.iter().copied()).collect()
}

fn gate_cargo_target_dir_with_explicit_path(cwd: &Path, explicit_path: Option<&Path>) -> PathBuf {
    explicit_path.map_or_else(|| cwd.join(DEFAULT_CRATE_SHAPE_GATE_TARGET_DIR), Path::to_path_buf)
}

fn gate_cargo_target_dir(cwd: &Path) -> PathBuf {
    gate_cargo_target_dir_with_explicit_path(
        cwd,
        std::env::var_os("CARGO_TARGET_DIR").as_deref().map(Path::new),
    )
}

fn build_benchmarking_ready_crate_shape_gate_report(
    cwd: &Path,
    output_path: &Path,
    cargo_target_dir: &Path,
    checks: Vec<CrateShapeGateCheckReport>,
) -> BenchmarkingReadyCrateShapeGateReport {
    let passed_goal_ids = checks
        .iter()
        .filter(|check| check.ok)
        .map(|check| check.goal_id.clone())
        .collect::<Vec<_>>();
    let failed_goal_ids = checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.goal_id.clone())
        .collect::<Vec<_>>();

    BenchmarkingReadyCrateShapeGateReport {
        schema_version: "bijux.crates.benchmarking_ready_gate.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        cargo_target_dir: relative_display(cargo_target_dir, cwd),
        ok: failed_goal_ids.is_empty(),
        checks,
        passed_goal_ids,
        failed_goal_ids,
    }
}

fn crate_shape_gate_check_from_result<R>(
    goal_id: &'static str,
    command: String,
    result: Result<R>,
    outcome: impl FnOnce(R) -> (bool, String),
) -> CrateShapeGateCheckReport {
    match result {
        Ok(report) => {
            let (ok, detail) = outcome(report);
            CrateShapeGateCheckReport { goal_id: goal_id.to_string(), ok, command, detail }
        }
        Err(err) => CrateShapeGateCheckReport {
            goal_id: goal_id.to_string(),
            ok: false,
            command,
            detail: err.to_string(),
        },
    }
}

fn collect_benchmarking_ready_crate_shape_gate_checks(
    cwd: &Path,
    cargo_target_dir: &Path,
) -> Vec<CrateShapeGateCheckReport> {
    let graph_path = cwd.join(DEFAULT_CRATE_DEPENDENCY_MAP_PATH);
    let goal_411 = crate_shape_gate_check_from_result(
        "411",
        format!("bijux-dna dev crates graph --output {DEFAULT_CRATE_DEPENDENCY_MAP_PATH}"),
        write_dependency_map(cwd, &graph_path),
        |report| (true, report.output_path),
    );

    let domain_no_execution_path = cwd.join(DEFAULT_DOMAIN_NO_EXECUTION_PATH);
    let goal_412 = crate_shape_gate_check_from_result(
        "412",
        format!(
            "bijux-dna dev crates domain-no-execution --output {DEFAULT_DOMAIN_NO_EXECUTION_PATH}"
        ),
        write_domain_no_execution_report(cwd, &domain_no_execution_path),
        |report| (report.ok, report.output_path),
    );

    let parser_no_execution_path = cwd.join(DEFAULT_PARSER_NO_EXECUTION_PATH);
    let goal_413 = crate_shape_gate_check_from_result(
        "413",
        format!(
            "bijux-dna dev crates parser-no-execution --output {DEFAULT_PARSER_NO_EXECUTION_PATH}"
        ),
        write_parser_no_execution_report(cwd, &parser_no_execution_path),
        |report| (report.ok, report.output_path),
    );

    let planner_no_parser_path = cwd.join(DEFAULT_PLANNER_NO_PARSER_PATH);
    let goal_414 = crate_shape_gate_check_from_result(
        "414",
        format!("bijux-dna dev crates planner-no-parser --output {DEFAULT_PLANNER_NO_PARSER_PATH}"),
        write_planner_no_parser_report(cwd, &planner_no_parser_path),
        |report| (report.ok, report.output_path),
    );

    let runner_process_execution_path = cwd.join(DEFAULT_RUNNER_OWNS_PROCESS_EXECUTION_PATH);
    let goal_415 = crate_shape_gate_check_from_result(
        "415",
        format!(
            "bijux-dna dev crates runner-owns-process-execution --output {DEFAULT_RUNNER_OWNS_PROCESS_EXECUTION_PATH}"
        ),
        write_runner_owned_process_execution_report(cwd, &runner_process_execution_path),
        |report| (report.ok, report.output_path),
    );

    let goal_416 = crate_shape_gate_check_from_result(
        "416",
        TYPED_ARTIFACT_HANDOFF_TEST_COMMAND.to_string(),
        run_gate_proof(
            cwd,
            cargo_target_dir,
            ProofId::TypedArtifactHandoff,
            TYPED_ARTIFACT_HANDOFF_TEST_COMMAND,
        ),
        |()| (true, "typed artifact handoff proof passed".to_string()),
    );

    let goal_417 = collect_metric_registry_gate_check(cwd, cargo_target_dir);

    let result_id_stability_path = cwd.join(DEFAULT_RESULT_ID_STABILITY_PATH);
    let goal_418 = crate_shape_gate_check_from_result(
        "418",
        format!(
            "bijux-dna dev crates result-id-stability --output {DEFAULT_RESULT_ID_STABILITY_PATH}"
        ),
        write_result_id_stability_report(cwd, &result_id_stability_path),
        |report| (report.ok, report.output_path),
    );

    let no_crate_cycles_path = cwd.join(DEFAULT_NO_CRATE_CYCLES_PATH);
    let goal_419 = crate_shape_gate_check_from_result(
        "419",
        format!("bijux-dna dev crates check-cycles --output {DEFAULT_NO_CRATE_CYCLES_PATH}"),
        write_no_crate_cycles_report(cwd, &no_crate_cycles_path),
        |report| (report.ok, report.output_path),
    );

    vec![goal_411, goal_412, goal_413, goal_414, goal_415, goal_416, goal_417, goal_418, goal_419]
}

fn collect_metric_registry_gate_check(
    cwd: &Path,
    cargo_target_dir: &Path,
) -> CrateShapeGateCheckReport {
    let metric_registry_path = cwd.join(DEFAULT_METRIC_REGISTRY_PATH);
    let metric_registry_report = write_metric_registry_report(cwd, &metric_registry_path);
    let metric_registry_contract = run_gate_proof(
        cwd,
        cargo_target_dir,
        ProofId::VcfMetricRegistry,
        METRIC_REGISTRY_PROOF_TEST_COMMAND,
    );
    match (metric_registry_report, metric_registry_contract) {
        (Ok(report), Ok(())) => CrateShapeGateCheckReport {
            goal_id: "417".to_string(),
            ok: true,
            command: format!(
                "bijux-dna dev crates metric-registry --output {DEFAULT_METRIC_REGISTRY_PATH} && {METRIC_REGISTRY_PROOF_TEST_COMMAND}"
            ),
            detail: report.output_path,
        },
        (Err(err), Ok(())) => CrateShapeGateCheckReport {
            goal_id: "417".to_string(),
            ok: false,
            command: format!(
                "bijux-dna dev crates metric-registry --output {DEFAULT_METRIC_REGISTRY_PATH} && {METRIC_REGISTRY_PROOF_TEST_COMMAND}"
            ),
            detail: err.to_string(),
        },
        (Ok(_report), Err(err)) => CrateShapeGateCheckReport {
            goal_id: "417".to_string(),
            ok: false,
            command: format!(
                "bijux-dna dev crates metric-registry --output {DEFAULT_METRIC_REGISTRY_PATH} && {METRIC_REGISTRY_PROOF_TEST_COMMAND}"
            ),
            detail: err.to_string(),
        },
        (Err(report_err), Err(test_err)) => CrateShapeGateCheckReport {
            goal_id: "417".to_string(),
            ok: false,
            command: format!(
                "bijux-dna dev crates metric-registry --output {DEFAULT_METRIC_REGISTRY_PATH} && {METRIC_REGISTRY_PROOF_TEST_COMMAND}"
            ),
            detail: format!("{report_err}; {test_err}"),
        },
    }
}

const MICRO_RESULT_ID_SUBSET: &[&str] = &[
    "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
    "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
    "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
];

fn result_scope_kind_label(
    kind: crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind,
) -> &'static str {
    match kind {
        crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind::SampleScope => {
            "sample_scope"
        }
        crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind::AssetProfile => {
            "asset_profile"
        }
    }
}

fn push_result_id_violation(
    violations: &mut Vec<ResultIdStabilityViolation>,
    result_id: &str,
    surface: &str,
    detail: impl Into<String>,
) {
    violations.push(ResultIdStabilityViolation {
        result_id: result_id.to_string(),
        surface: surface.to_string(),
        detail: detail.into(),
    });
}

fn collect_canonical_result_id_rows(
    cwd: &Path,
) -> Result<(Vec<CanonicalResultIdRow>, Vec<ResultIdStabilityViolation>)> {
    let mut violations = Vec::new();
    let mut rows = collect_sample_scoped_canonical_result_id_rows(cwd, &mut violations)?;
    rows.extend(collect_asset_profile_canonical_result_id_rows(cwd)?);
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    validate_canonical_result_id_rows(cwd, &rows, &mut violations)?;

    Ok((rows, violations))
}

fn collect_sample_scoped_canonical_result_id_rows(
    cwd: &Path,
    violations: &mut Vec<ResultIdStabilityViolation>,
) -> Result<Vec<CanonicalResultIdRow>> {
    let mut rows = Vec::new();
    for row in crate::commands::benchmark::readiness::expected_benchmark_results::collect_expected_benchmark_result_rows(cwd)?
    {
        let canonical_result_id = crate::commands::benchmark::benchmark_result_ids::build_sample_scoped_benchmark_result_id(
            &row.domain,
            &row.fixture_id,
            &row.stage_id,
            &row.sample_scope,
            &row.tool_id,
        );
        if canonical_result_id != row.result_row_id {
            push_result_id_violation(
                violations,
                &row.result_row_id,
                "expected_benchmark_results",
                format!(
                    "FASTQ/BAM expected row drifted from the canonical sample-scoped builder: `{canonical_result_id}`"
                ),
            );
        }
        rows.push(CanonicalResultIdRow {
            result_id: canonical_result_id,
            domain: row.domain,
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            corpus_id: row.fixture_id,
            scope_kind: crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind::SampleScope,
            scope_id: row.sample_scope,
        });
    }
    Ok(rows)
}

fn collect_asset_profile_canonical_result_id_rows(cwd: &Path) -> Result<Vec<CanonicalResultIdRow>> {
    let mut rows = Vec::new();
    for row in crate::commands::benchmark::readiness::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows(cwd)?
    {
        let canonical_result_id = crate::commands::benchmark::benchmark_result_ids::build_asset_profile_benchmark_result_id(
            &row.domain,
            &row.corpus_id,
            &row.stage_id,
            &row.asset_profile_id,
            &row.tool_id,
        );
        rows.push(CanonicalResultIdRow {
            result_id: canonical_result_id,
            domain: row.domain,
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            corpus_id: row.corpus_id,
            scope_kind: crate::commands::benchmark::benchmark_result_ids::BenchmarkResultScopeKind::AssetProfile,
            scope_id: row.asset_profile_id,
        });
    }
    Ok(rows)
}

fn validate_canonical_result_id_rows(
    cwd: &Path,
    rows: &[CanonicalResultIdRow],
    violations: &mut Vec<ResultIdStabilityViolation>,
) -> Result<()> {
    let mut identity_violations = Vec::new();
    let canonical = index_canonical_benchmark_result_bindings(rows, &mut identity_violations);
    violations.extend(identity_violations.drain(..).map(ResultIdStabilityViolation::from));

    let all_domain_rows = crate::commands::benchmark::readiness::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows(cwd)?;
    let result = reconcile_canonical_benchmark_result_identities(
        &canonical,
        &all_domain_rows,
        &mut identity_violations,
    );
    violations.extend(identity_violations.into_iter().map(ResultIdStabilityViolation::from));
    result.map_err(Into::into)
}

fn validate_optional_result_id_surface<T>(
    violations: &mut Vec<ResultIdStabilityViolation>,
    canonical_result_id: &str,
    surface: &'static str,
    row: Option<&T>,
    missing_detail: &'static str,
    validate: impl FnOnce(&T) -> Option<String>,
) {
    match row {
        Some(row) => {
            if let Some(detail) = validate(row) {
                push_result_id_violation(violations, canonical_result_id, surface, detail);
            }
        }
        None => {
            push_result_id_violation(violations, canonical_result_id, surface, missing_detail);
        }
    }
}

macro_rules! collect_result_id_stability_rows {
    (
        $violations:expr,
        $canonical_rows:expr,
        $canonical_by_result:expr,
        $local_rows:expr,
        $local_by_result:expr,
        $fake_report:expr,
        $fake_by_result:expr,
        $full_report:expr,
        $report_by_result:expr,
        $slurm_report:expr,
        $slurm_by_result:expr,
        $micro_result_ids:expr
    ) => {{
        for result_id in $local_by_result.keys() {
            if !$canonical_by_result.contains_key(result_id) {
                push_result_id_violation(
                    $violations,
                    result_id,
                    "local_all_domain_rendered_commands",
                    "local execution surface produced a non-canonical result id",
                );
            }
        }
        for result_id in $fake_by_result.keys() {
            if !$canonical_by_result.contains_key(result_id) {
                push_result_id_violation(
                    $violations,
                    result_id,
                    "local_all_domain_fake_runs",
                    "fake-run surface produced a non-canonical result id",
                );
            }
        }
        for result_id in $report_by_result.keys() {
            if !$canonical_by_result.contains_key(result_id) {
                push_result_id_violation(
                    $violations,
                    result_id,
                    "full_benchmark_report",
                    "full benchmark report produced a non-canonical result id",
                );
            }
        }
        for result_id in $slurm_by_result.keys() {
            if !$canonical_by_result.contains_key(result_id) {
                push_result_id_violation(
                    $violations,
                    result_id,
                    "local_all_domain_slurm_submit_manifest",
                    "SLURM dry-run surface produced a non-canonical result id",
                );
            }
        }
        for result_id in $micro_result_ids {
            if !$canonical_by_result.contains_key(result_id) {
                push_result_id_violation(
                    $violations,
                    result_id,
                    "governed_micro_subset",
                    "micro result-id subset references a non-canonical benchmark row",
                );
            }
        }

        let mut rows = Vec::with_capacity($canonical_rows.len());
        for canonical in $canonical_rows {
            let local_row = $local_by_result.get(&canonical.result_id).copied();
            let fake_row = $fake_by_result.get(&canonical.result_id).copied();
            let report_row = $report_by_result.get(&canonical.result_id).copied();
            let slurm_job = $slurm_by_result.get(&canonical.result_id).copied();

            validate_optional_result_id_surface(
                $violations,
                &canonical.result_id,
                "local_all_domain_rendered_commands",
                local_row,
                "local execution surface is missing the canonical result id",
                |row| match crate::commands::benchmark::local_all_domain_result_paths::benchmark_result_matches_identity(
                    &row.result_id,
                    &canonical.domain,
                    &canonical.corpus_id,
                    &canonical.stage_id,
                    &canonical.tool_id,
                ) {
                    Ok(true) => None,
                    Ok(false) => Some(
                        "local execution row drifted from the canonical benchmark identity"
                            .to_string(),
                    ),
                    Err(err) => Some(err.to_string()),
                },
            );
            validate_optional_result_id_surface(
                $violations,
                &canonical.result_id,
                "local_all_domain_fake_runs",
                fake_row,
                "fake-run surface is missing the canonical result id",
                |row| match crate::commands::benchmark::local_all_domain_result_paths::benchmark_result_matches_identity(
                    &row.result_id,
                    &canonical.domain,
                    &canonical.corpus_id,
                    &canonical.stage_id,
                    &canonical.tool_id,
                ) {
                    Ok(true) => None,
                    Ok(false) => Some(
                        "fake-run row drifted from the canonical benchmark identity".to_string(),
                    ),
                    Err(err) => Some(err.to_string()),
                },
            );
            validate_optional_result_id_surface(
                $violations,
                &canonical.result_id,
                "full_benchmark_report",
                report_row,
                "full benchmark report is missing the canonical result id",
                |row| {
                    if row.report_row_id != canonical.result_id {
                        return Some(format!(
                            "full benchmark report row id `{}` drifted from canonical result id",
                            row.report_row_id
                        ));
                    }
                    if row.domain != canonical.domain
                        || row.stage_id != canonical.stage_id
                        || row.tool_id != canonical.tool_id
                        || row.corpus_id != canonical.corpus_id
                    {
                        return Some(
                            "full benchmark report row drifted from the canonical benchmark identity"
                                .to_string(),
                        );
                    }
                    None
                },
            );
            validate_optional_result_id_surface(
                $violations,
                &canonical.result_id,
                "local_all_domain_slurm_submit_manifest",
                slurm_job,
                "SLURM dry-run surface is missing the canonical result id",
                |job| {
                    if job.job_id_local != format!("benchmark:{}", canonical.result_id) {
                        return Some(format!(
                            "SLURM dry-run job id `{}` drifted from the canonical benchmark job prefix",
                            job.job_id_local
                        ));
                    }
                    if job.domain != canonical.domain
                        || job.stage_id != canonical.stage_id
                        || job.tool_id != canonical.tool_id
                        || job.corpus_id != canonical.corpus_id
                    {
                        return Some(
                            "SLURM dry-run job drifted from the canonical benchmark identity"
                                .to_string(),
                        );
                    }
                    None
                },
            );

            let local_execution_argv = crate::commands::benchmark::local_all_domain_job_execution::rendered_benchmark_result_execution_argv(&canonical.result_id);
            let selected_for_micro = $micro_result_ids.contains(&canonical.result_id);

            rows.push(ResultIdStabilityRow {
                result_id: canonical.result_id.clone(),
                domain: canonical.domain.clone(),
                stage_id: canonical.stage_id.clone(),
                tool_id: canonical.tool_id.clone(),
                corpus_id: canonical.corpus_id.clone(),
                scope_kind: result_scope_kind_label(canonical.scope_kind).to_string(),
                scope_id: canonical.scope_id.clone(),
                local_result_id: local_row.map(|row| row.result_id.clone()),
                fake_result_id: fake_row.map(|row| row.result_id.clone()),
                micro_result_id: selected_for_micro.then(|| canonical.result_id.clone()),
                report_result_id: report_row.and_then(|row| row.result_id.clone()),
                slurm_result_id: slurm_job.and_then(|job| job.result_id.clone()),
                local_execution_argv: Some(local_execution_argv.clone()),
                micro_execution_argv: selected_for_micro.then_some(local_execution_argv),
                fake_metrics_path: fake_row.map(|row| row.metrics_path.clone()),
                report_row_id: report_row.map(|row| row.report_row_id.clone()),
                report_evidence_path: report_row.map(|row| row.evidence_path.clone()),
                slurm_job_id_local: slurm_job.map(|job| job.job_id_local.clone()),
            });
        }
        rows
    }};
}

fn collect_rust_sources(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, files)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn collect_rust_sources_with_exclusions(
    root: &Path,
    current: &Path,
    excluded_paths: &BTreeMap<PathBuf, String>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if !current.exists() {
        return Ok(());
    }
    if excluded_paths.contains_key(&current.to_path_buf()) {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(current).with_context(|| format!("read {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if excluded_paths.contains_key(&path) {
            continue;
        }
        if path.is_dir() {
            collect_rust_sources_with_exclusions(root, &path, excluded_paths, files)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
    Ok(())
}

fn push_source_hits(
    hits: &mut Vec<SourcePatternHit>,
    path: &Path,
    root: &Path,
    content: &str,
    patterns: &[&str],
) {
    for (line_number, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        for pattern in patterns {
            if line.contains(pattern) {
                hits.push(SourcePatternHit {
                    path: relative_display(path, root),
                    line: line_number + 1,
                    pattern: (*pattern).to_string(),
                });
            }
        }
    }
}

fn collect_manifest_dependency_hits(
    manifest: &Value,
    section_name: &str,
    patterns: &[&str],
    category: &str,
) -> Vec<ForbiddenDependencyHit> {
    let Some(table) = manifest.get(section_name).and_then(Value::as_table) else {
        return Vec::new();
    };
    let mut hits = table
        .keys()
        .filter(|dependency| {
            let normalized = dependency.to_ascii_lowercase();
            patterns.iter().any(|pattern| normalized.contains(pattern))
        })
        .map(|dependency| ForbiddenDependencyHit {
            section: section_name.to_string(),
            dependency: dependency.clone(),
            category: category.to_string(),
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| left.dependency.cmp(&right.dependency));
    hits
}

fn audit_domain_crate(
    cwd: &Path,
    member: &WorkspaceMember,
) -> Result<DomainNoExecutionCrateReport> {
    let crate_root = member
        .manifest_path
        .parent()
        .with_context(|| format!("resolve crate root from {}", member.manifest_path.display()))?;
    let manifest_text = std::fs::read_to_string(&member.manifest_path)
        .with_context(|| format!("read {}", member.manifest_path.display()))?;
    let manifest_value: Value = toml::from_str(&manifest_text)
        .with_context(|| format!("parse {}", member.manifest_path.display()))?;

    let mut rust_files = Vec::new();
    collect_rust_sources(&crate_root.join("src"), &mut rust_files)?;
    let build_rs = crate_root.join("build.rs");
    if build_rs.is_file() {
        rust_files.push(build_rs);
    }
    rust_files.sort();

    let mut process_execution_refs = Vec::new();
    let mut container_execution_refs = Vec::new();
    let mut slurm_execution_refs = Vec::new();
    for rust_file in &rust_files {
        let content = std::fs::read_to_string(rust_file)
            .with_context(|| format!("read {}", rust_file.display()))?;
        push_source_hits(
            &mut process_execution_refs,
            rust_file,
            cwd,
            &content,
            PROCESS_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut container_execution_refs,
            rust_file,
            cwd,
            &content,
            CONTAINER_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut slurm_execution_refs,
            rust_file,
            cwd,
            &content,
            SLURM_EXECUTION_PATTERNS,
        );
    }

    let mut forbidden_direct_dependencies = Vec::new();
    for (patterns, category) in [
        (RUNNER_DEPENDENCY_PATTERNS, "runner"),
        (CONTAINER_DEPENDENCY_PATTERNS, "container"),
        (SLURM_DEPENDENCY_PATTERNS, "slurm"),
    ] {
        for section_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
            forbidden_direct_dependencies.extend(collect_manifest_dependency_hits(
                &manifest_value,
                section_name,
                patterns,
                category,
            ));
        }
    }
    forbidden_direct_dependencies.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.section.cmp(&right.section))
            .then_with(|| left.dependency.cmp(&right.dependency))
    });

    let scanned_rust_files =
        rust_files.iter().map(|path| relative_display(path, cwd)).collect::<Vec<_>>();

    let ok = forbidden_direct_dependencies.is_empty()
        && process_execution_refs.is_empty()
        && container_execution_refs.is_empty()
        && slurm_execution_refs.is_empty();

    Ok(DomainNoExecutionCrateReport {
        crate_name: member.crate_name.clone(),
        manifest_path: relative_display(&member.manifest_path, cwd),
        scanned_rust_files,
        forbidden_direct_dependencies,
        process_execution_refs,
        container_execution_refs,
        slurm_execution_refs,
        ok,
    })
}

fn parser_surface_spec(crate_name: &str) -> Option<&'static ParserSurfaceSpec> {
    PARSER_SURFACES.iter().find(|spec| spec.crate_name == crate_name)
}

fn planner_audit_spec(crate_name: &str) -> Option<&'static PlannerCrateAuditSpec> {
    PLANNER_AUDIT_SPECS.iter().find(|spec| spec.crate_name == crate_name)
}

fn runner_owned_process_execution_category(crate_name: &str) -> Option<&'static str> {
    if crate_name.starts_with("bijux-dna-domain-") {
        return Some("domain");
    }
    if crate_name.starts_with("bijux-dna-stages-") {
        return Some("stage");
    }
    if matches!(crate_name, "bijux-dna-analyze" | "bijux-dna-bench-model") {
        return Some("report");
    }
    None
}

fn audit_parser_surface(
    cwd: &Path,
    member: &WorkspaceMember,
    spec: &ParserSurfaceSpec,
) -> Result<ParserSurfaceAuditReport> {
    let crate_root = member
        .manifest_path
        .parent()
        .with_context(|| format!("resolve crate root from {}", member.manifest_path.display()))?;
    let manifest_text = std::fs::read_to_string(&member.manifest_path)
        .with_context(|| format!("read {}", member.manifest_path.display()))?;
    let manifest_value: Value = toml::from_str(&manifest_text)
        .with_context(|| format!("parse {}", member.manifest_path.display()))?;

    let parser_root = crate_root.join(spec.parser_root);
    let excluded_paths = spec
        .excluded_paths
        .iter()
        .map(|excluded| (crate_root.join(excluded.path), excluded.reason.to_string()))
        .collect::<BTreeMap<_, _>>();

    let mut rust_files = Vec::new();
    collect_rust_sources_with_exclusions(
        crate_root,
        &parser_root,
        &excluded_paths,
        &mut rust_files,
    )?;
    rust_files.sort();

    let excluded_governance_paths = spec
        .excluded_paths
        .iter()
        .filter_map(|excluded| {
            let absolute = crate_root.join(excluded.path);
            if absolute.exists() {
                Some(ExcludedAuditPath {
                    path: relative_display(&absolute, cwd),
                    reason: excluded.reason.to_string(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let (
        parser_entrypoints,
        raw_input_read_refs,
        input_mutation_refs,
        process_execution_refs,
        container_execution_refs,
        slurm_execution_refs,
    ) = collect_parser_surface_hits(cwd, crate_root, &rust_files)?;
    let forbidden_direct_dependencies = collect_forbidden_direct_dependencies(&manifest_value);
    let scanned_rust_files = rust_files
        .iter()
        .map(|path| relative_display(&crate_root.join(path), cwd))
        .collect::<Vec<_>>();
    let ok = !parser_entrypoints.is_empty()
        && forbidden_direct_dependencies.is_empty()
        && input_mutation_refs.is_empty()
        && process_execution_refs.is_empty()
        && container_execution_refs.is_empty()
        && slurm_execution_refs.is_empty();

    Ok(ParserSurfaceAuditReport {
        crate_name: member.crate_name.clone(),
        manifest_path: relative_display(&member.manifest_path, cwd),
        parser_root: relative_display(&parser_root, cwd),
        scanned_rust_files,
        excluded_governance_paths,
        parser_entrypoints,
        raw_input_read_refs,
        input_mutation_refs,
        forbidden_direct_dependencies,
        process_execution_refs,
        container_execution_refs,
        slurm_execution_refs,
        ok,
    })
}

fn collect_parser_surface_hits(
    cwd: &Path,
    crate_root: &Path,
    rust_files: &[PathBuf],
) -> Result<ParserSurfaceHits> {
    let mut parser_entrypoints = Vec::new();
    let mut raw_input_read_refs = Vec::new();
    let mut input_mutation_refs = Vec::new();
    let mut process_execution_refs = Vec::new();
    let mut container_execution_refs = Vec::new();
    let mut slurm_execution_refs = Vec::new();
    for rust_file in rust_files {
        let absolute = crate_root.join(rust_file);
        let content = std::fs::read_to_string(&absolute)
            .with_context(|| format!("read {}", absolute.display()))?;
        push_source_hits(
            &mut parser_entrypoints,
            &absolute,
            cwd,
            &content,
            PARSER_ENTRYPOINT_PATTERNS,
        );
        push_source_hits(
            &mut raw_input_read_refs,
            &absolute,
            cwd,
            &content,
            RAW_INPUT_READ_PATTERNS,
        );
        push_source_hits(
            &mut input_mutation_refs,
            &absolute,
            cwd,
            &content,
            INPUT_MUTATION_PATTERNS,
        );
        push_source_hits(
            &mut process_execution_refs,
            &absolute,
            cwd,
            &content,
            PROCESS_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut container_execution_refs,
            &absolute,
            cwd,
            &content,
            CONTAINER_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut slurm_execution_refs,
            &absolute,
            cwd,
            &content,
            SLURM_EXECUTION_PATTERNS,
        );
    }
    Ok((
        parser_entrypoints,
        raw_input_read_refs,
        input_mutation_refs,
        process_execution_refs,
        container_execution_refs,
        slurm_execution_refs,
    ))
}

fn collect_forbidden_direct_dependencies(manifest_value: &Value) -> Vec<ForbiddenDependencyHit> {
    let mut forbidden_direct_dependencies = Vec::new();
    for (patterns, category) in [
        (RUNNER_DEPENDENCY_PATTERNS, "runner"),
        (CONTAINER_DEPENDENCY_PATTERNS, "container"),
        (SLURM_DEPENDENCY_PATTERNS, "slurm"),
    ] {
        for section_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
            forbidden_direct_dependencies.extend(collect_manifest_dependency_hits(
                manifest_value,
                section_name,
                patterns,
                category,
            ));
        }
    }
    forbidden_direct_dependencies.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.section.cmp(&right.section))
            .then_with(|| left.dependency.cmp(&right.dependency))
    });
    forbidden_direct_dependencies
}

fn audit_planner_crate(
    cwd: &Path,
    member: &WorkspaceMember,
    spec: &PlannerCrateAuditSpec,
) -> Result<PlannerNoParserCrateReport> {
    let crate_root = member
        .manifest_path
        .parent()
        .with_context(|| format!("resolve crate root from {}", member.manifest_path.display()))?;

    let mut rust_files = Vec::new();
    collect_rust_sources(&crate_root.join("src"), &mut rust_files)?;
    rust_files.sort();

    let allowed_input_read_paths = spec
        .allowed_input_read_files
        .iter()
        .map(|allowed| AllowedAuditPath {
            path: allowed.path.to_string(),
            reason: allowed.reason.to_string(),
        })
        .collect::<Vec<_>>();
    let allowed_input_read_files =
        spec.allowed_input_read_files.iter().map(|allowed| allowed.path).collect::<BTreeSet<_>>();

    let mut planner_input_read_refs = Vec::new();
    let mut unexpected_input_read_refs = Vec::new();
    let mut forbidden_parser_api_refs = Vec::new();
    for rust_file in &rust_files {
        let content = std::fs::read_to_string(rust_file)
            .with_context(|| format!("read {}", rust_file.display()))?;
        let mut read_refs = Vec::new();
        push_source_hits(&mut read_refs, rust_file, cwd, &content, RAW_INPUT_READ_PATTERNS);
        for hit in read_refs {
            if allowed_input_read_files.contains(hit.path.as_str()) {
                planner_input_read_refs.push(hit);
            } else {
                unexpected_input_read_refs.push(hit);
            }
        }
        push_source_hits(
            &mut forbidden_parser_api_refs,
            rust_file,
            cwd,
            &content,
            PLANNER_PARSER_API_PATTERNS,
        );
    }

    let scanned_rust_files =
        rust_files.iter().map(|path| relative_display(path, cwd)).collect::<Vec<_>>();
    let ok = unexpected_input_read_refs.is_empty() && forbidden_parser_api_refs.is_empty();

    Ok(PlannerNoParserCrateReport {
        crate_name: member.crate_name.clone(),
        manifest_path: relative_display(&member.manifest_path, cwd),
        scanned_rust_files,
        allowed_input_read_paths,
        planner_input_read_refs,
        unexpected_input_read_refs,
        forbidden_parser_api_refs,
        ok,
    })
}

fn collect_execution_owner_dependency_hits(
    manifest: &Value,
    section_name: &str,
) -> Vec<ExecutionOwnerDependencyHit> {
    let Some(table) = manifest.get(section_name).and_then(Value::as_table) else {
        return Vec::new();
    };
    let mut hits = table
        .keys()
        .filter(|dependency| {
            matches!(dependency.as_str(), "bijux-dna-runner" | "bijux-dna-runtime")
        })
        .map(|dependency| ExecutionOwnerDependencyHit {
            section: section_name.to_string(),
            dependency: dependency.clone(),
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        left.section.cmp(&right.section).then_with(|| left.dependency.cmp(&right.dependency))
    });
    hits
}

fn audit_runner_owned_process_execution_crate(
    cwd: &Path,
    member: &WorkspaceMember,
) -> Result<RunnerOwnedProcessExecutionCrateReport> {
    let crate_root = member
        .manifest_path
        .parent()
        .with_context(|| format!("resolve crate root from {}", member.manifest_path.display()))?;
    let manifest_text = std::fs::read_to_string(&member.manifest_path)
        .with_context(|| format!("read {}", member.manifest_path.display()))?;
    let manifest_value: Value = toml::from_str(&manifest_text)
        .with_context(|| format!("parse {}", member.manifest_path.display()))?;

    let mut rust_files = Vec::new();
    collect_rust_sources(&crate_root.join("src"), &mut rust_files)?;
    let build_rs = crate_root.join("build.rs");
    if build_rs.is_file() {
        rust_files.push(build_rs);
    }
    rust_files.sort();

    let mut process_execution_refs = Vec::new();
    let mut container_execution_refs = Vec::new();
    let mut slurm_execution_refs = Vec::new();
    for rust_file in &rust_files {
        let content = std::fs::read_to_string(rust_file)
            .with_context(|| format!("read {}", rust_file.display()))?;
        push_source_hits(
            &mut process_execution_refs,
            rust_file,
            cwd,
            &content,
            PROCESS_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut container_execution_refs,
            rust_file,
            cwd,
            &content,
            CONTAINER_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut slurm_execution_refs,
            rust_file,
            cwd,
            &content,
            SLURM_EXECUTION_PATTERNS,
        );
    }

    let mut execution_owner_dependencies = Vec::new();
    for section_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        execution_owner_dependencies
            .extend(collect_execution_owner_dependency_hits(&manifest_value, section_name));
    }
    execution_owner_dependencies.sort_by(|left, right| {
        left.section.cmp(&right.section).then_with(|| left.dependency.cmp(&right.dependency))
    });

    let scanned_rust_files =
        rust_files.iter().map(|path| relative_display(path, cwd)).collect::<Vec<_>>();
    let ok = process_execution_refs.is_empty()
        && container_execution_refs.is_empty()
        && slurm_execution_refs.is_empty();

    Ok(RunnerOwnedProcessExecutionCrateReport {
        crate_name: member.crate_name.clone(),
        category: runner_owned_process_execution_category(&member.crate_name)
            .unwrap_or("unclassified")
            .to_string(),
        manifest_path: relative_display(&member.manifest_path, cwd),
        scanned_rust_files,
        execution_owner_dependencies,
        process_execution_refs,
        container_execution_refs,
        slurm_execution_refs,
        ok,
    })
}

/// # Errors
/// Returns an error if the workspace crate graph cannot be resolved or if an audited cycle exists.
pub fn write_no_crate_cycles_report(cwd: &Path, output_path: &Path) -> Result<NoCrateCyclesReport> {
    let members = workspace_members(cwd)?;
    let member_lookup = members
        .iter()
        .map(|member| (member.crate_name.clone(), member.manifest_path.clone()))
        .collect::<BTreeMap<_, _>>();
    let member_names = member_lookup.keys().cloned().collect::<BTreeSet<_>>();
    let audited_names = audited_crate_cycle_names();
    let configured_names =
        audited_names.iter().map(|name| (*name).to_string()).collect::<BTreeSet<_>>();

    let missing_crates = configured_names.difference(&member_names).cloned().collect::<Vec<_>>();
    if !missing_crates.is_empty() {
        bail!(
            "crate cycle audit configuration references workspace crates that do not exist: {}",
            missing_crates.join(", ")
        );
    }

    let edges = bijux_dna_api::v1::api::workspace_edges_at(cwd).context("load workspace edges")?;
    let mut direct_deps = configured_names
        .iter()
        .map(|crate_name| (crate_name.clone(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    let mut direct_dependents = configured_names
        .iter()
        .map(|crate_name| (crate_name.clone(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    let mut edge_rows = Vec::new();
    for (from, to) in edges {
        if !configured_names.contains(&from) || !configured_names.contains(&to) {
            continue;
        }
        direct_deps.entry(from.clone()).or_default().insert(to.clone());
        direct_dependents.entry(to.clone()).or_default().insert(from.clone());
        edge_rows.push(CrateDependencyEdge { from, to });
    }
    edge_rows
        .sort_by(|left, right| left.from.cmp(&right.from).then_with(|| left.to.cmp(&right.to)));

    let categories = collect_no_crate_cycle_categories();
    let crates = collect_no_crate_cycle_crates(
        cwd,
        &configured_names,
        &member_lookup,
        &direct_deps,
        &direct_dependents,
    )?;
    let analysis = bijux_dna_api::v1::api::analyze_workspace_dependencies(&direct_deps)?;
    let cycles = analysis
        .cycles
        .into_iter()
        .map(|crates| NoCrateCyclesComponent { crates })
        .collect::<Vec<_>>();
    let topological_order = analysis.dependency_order;
    let skipped_workspace_crates =
        member_names.difference(&configured_names).cloned().collect::<Vec<_>>();
    let report = NoCrateCyclesReport {
        schema_version: "bijux.crates.no_crate_cycles.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        audited_crate_count: crates.len(),
        skipped_workspace_crate_count: skipped_workspace_crates.len(),
        category_count: categories.len(),
        edge_count: edge_rows.len(),
        cycle_count: cycles.len(),
        ok: cycles.is_empty(),
        categories,
        crates,
        edges: edge_rows,
        cycles,
        topological_order,
        skipped_workspace_crates,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    if !report.ok {
        bail!("crate dependency cycles detected among audited crates; see {}", report.output_path);
    }
    Ok(report)
}

fn collect_no_crate_cycle_categories() -> Vec<NoCrateCyclesCategoryReport> {
    CRATE_CYCLE_CATEGORIES
        .iter()
        .map(|spec| NoCrateCyclesCategoryReport {
            category: spec.category.to_string(),
            crate_count: spec.crates.len(),
            crates: spec.crates.iter().map(|crate_name| (*crate_name).to_string()).collect(),
        })
        .collect()
}

fn collect_no_crate_cycle_crates(
    cwd: &Path,
    configured_names: &BTreeSet<String>,
    member_lookup: &BTreeMap<String, PathBuf>,
    direct_deps: &BTreeMap<String, BTreeSet<String>>,
    direct_dependents: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<NoCrateCyclesCrateReport>> {
    configured_names
        .iter()
        .map(|crate_name| {
            let Some(manifest_path) = member_lookup.get(crate_name) else {
                return Err(anyhow!("audited crate manifest path must exist for `{crate_name}`"));
            };
            Ok(NoCrateCyclesCrateReport {
                crate_name: crate_name.clone(),
                category: crate_cycle_category(crate_name).unwrap_or("unclassified").to_string(),
                manifest_path: relative_display(manifest_path, cwd),
                direct_workspace_dependencies: direct_deps
                    .get(crate_name)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                direct_workspace_dependents: direct_dependents
                    .get(crate_name)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
            })
        })
        .collect()
}

/// # Errors
/// Returns an error if any crate-shape proof for Goals 411-419 fails.
pub fn write_benchmarking_ready_crate_shape_gate_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<BenchmarkingReadyCrateShapeGateReport> {
    let cargo_target_dir = gate_cargo_target_dir(cwd);
    let checks = collect_benchmarking_ready_crate_shape_gate_checks(cwd, &cargo_target_dir);
    let report = build_benchmarking_ready_crate_shape_gate_report(
        cwd,
        output_path,
        &cargo_target_dir,
        checks,
    );

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    if !report.ok {
        bail!("benchmarking-ready crate-shape gate failed; see {}", output_path.display());
    }
    Ok(report)
}

/// # Errors
/// Returns an error if the benchmark result-id stability audit cannot be resolved or written.
pub fn write_result_id_stability_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<ResultIdStabilityReport> {
    let (canonical_rows, mut violations) = collect_canonical_result_id_rows(cwd)?;
    let canonical_by_result =
        canonical_rows.iter().map(|row| (row.result_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let local_rows = crate::commands::benchmark::readiness::all_domain_rendered_commands::collect_all_domain_rendered_command_rows(cwd)?;
    let local_by_result =
        local_rows.iter().map(|row| (row.result_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let fake_report = crate::commands::benchmark::local_all_domain_fake_runs::fake_run_all_domain_benchmark_results(
        cwd,
        PathBuf::from(crate::commands::benchmark::local_all_domain_fake_runs::DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT),
    )?;
    let fake_by_result = fake_report
        .results
        .iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let full_report = crate::commands::benchmark::readiness::full_benchmark_report::render_full_benchmark_report(
        cwd,
        PathBuf::from(crate::commands::benchmark::readiness::full_benchmark_report::DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH),
    )?;
    let report_by_result = full_report
        .rows
        .iter()
        .filter_map(|row| row.result_id.as_ref().map(|result_id| (result_id.clone(), row)))
        .collect::<BTreeMap<_, _>>();

    let slurm_report = crate::commands::benchmark::local_all_domain_slurm_submit_manifest::render_all_domain_slurm_submit_manifest(
        cwd,
        PathBuf::from(crate::commands::benchmark::local_all_domain_slurm_scripts::DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
        PathBuf::from(crate::commands::benchmark::local_all_domain_slurm_submit_manifest::DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH),
    )?;
    let slurm_by_result = slurm_report
        .jobs
        .iter()
        .filter_map(|job| job.result_id.as_ref().map(|result_id| (result_id.clone(), job)))
        .collect::<BTreeMap<_, _>>();
    let micro_result_ids = MICRO_RESULT_ID_SUBSET
        .iter()
        .map(|result_id| (*result_id).to_string())
        .collect::<BTreeSet<_>>();
    let rows = collect_result_id_stability_rows!(
        &mut violations,
        &canonical_rows,
        &canonical_by_result,
        &local_rows,
        &local_by_result,
        &fake_report,
        &fake_by_result,
        &full_report,
        &report_by_result,
        &slurm_report,
        &slurm_by_result,
        &micro_result_ids
    );

    let report = ResultIdStabilityReport {
        schema_version: "bijux.crates.result_id_stability.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        row_count: rows.len(),
        local_row_count: local_rows.len(),
        fake_row_count: fake_report.result_count,
        report_row_count: full_report.rows.iter().filter(|row| row.result_id.is_some()).count(),
        slurm_row_count: slurm_report.jobs.iter().filter(|job| job.result_id.is_some()).count(),
        micro_checked_row_count: rows.iter().filter(|row| row.micro_result_id.is_some()).count(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}

/// # Errors
/// Returns an error if the domain crate execution audit cannot be resolved or written.
pub fn write_domain_no_execution_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<DomainNoExecutionReport> {
    let members = workspace_members(cwd)?;
    let mut crates = members
        .iter()
        .filter(|member| DOMAIN_CRATES.contains(&member.crate_name.as_str()))
        .map(|member| audit_domain_crate(cwd, member))
        .collect::<Result<Vec<_>>>()?;
    crates.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));

    let report = DomainNoExecutionReport {
        schema_version: "bijux.crates.domain_no_execution.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        audited_crate_count: crates.len(),
        ok: crates.iter().all(|crate_report| crate_report.ok),
        crates,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}

/// # Errors
/// Returns an error if the parser surface execution audit cannot be resolved or written.
pub fn write_parser_no_execution_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<ParserNoExecutionReport> {
    let members = workspace_members(cwd)?;
    let mut surfaces = members
        .iter()
        .filter_map(|member| parser_surface_spec(&member.crate_name).map(|spec| (member, spec)))
        .map(|(member, spec)| audit_parser_surface(cwd, member, spec))
        .collect::<Result<Vec<_>>>()?;
    surfaces.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));

    let report = ParserNoExecutionReport {
        schema_version: "bijux.crates.parser_no_execution.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        audited_surface_count: surfaces.len(),
        ok: surfaces.iter().all(|surface| surface.ok),
        surfaces,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}

/// # Errors
/// Returns an error if the planner crate parsing audit cannot be resolved or written.
pub fn write_planner_no_parser_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<PlannerNoParserReport> {
    let members = workspace_members(cwd)?;
    let mut crates = members
        .iter()
        .filter(|member| PLANNER_CRATES.contains(&member.crate_name.as_str()))
        .filter_map(|member| planner_audit_spec(&member.crate_name).map(|spec| (member, spec)))
        .map(|(member, spec)| audit_planner_crate(cwd, member, spec))
        .collect::<Result<Vec<_>>>()?;
    crates.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));

    let report = PlannerNoParserReport {
        schema_version: "bijux.crates.planner_no_parser.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        audited_crate_count: crates.len(),
        ok: crates.iter().all(|crate_report| crate_report.ok),
        crates,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}

/// # Errors
/// Returns an error if the runner-owned execution audit cannot be resolved or written.
pub fn write_runner_owned_process_execution_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<RunnerOwnedProcessExecutionReport> {
    let members = workspace_members(cwd)?;
    let mut crates = members
        .iter()
        .filter(|member| {
            RUNNER_OWNED_PROCESS_EXECUTION_REPORT_CRATES.contains(&member.crate_name.as_str())
        })
        .filter(|member| runner_owned_process_execution_category(&member.crate_name).is_some())
        .map(|member| audit_runner_owned_process_execution_crate(cwd, member))
        .collect::<Result<Vec<_>>>()?;
    crates.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));

    let report = RunnerOwnedProcessExecutionReport {
        schema_version: "bijux.crates.runner_owned_process_execution.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        audited_crate_count: crates.len(),
        ok: crates.iter().all(|crate_report| crate_report.ok),
        crates,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::{
        build_benchmarking_ready_crate_shape_gate_report, gate_cargo_target_dir_with_explicit_path,
        CrateShapeGateCheckReport, CRATE_CYCLE_CATEGORIES, CRATE_CYCLE_CLI_CRATES,
        CRATE_CYCLE_DOMAIN_CRATES,
    };
    use std::path::Path;

    #[test]
    fn crate_cycle_category_configuration_keeps_cli_and_domain_scopes_explicit() {
        assert_eq!(CRATE_CYCLE_CATEGORIES.len(), 7);
        assert_eq!(
            CRATE_CYCLE_DOMAIN_CRATES,
            &[
                "bijux-dna-domain-bam",
                "bijux-dna-domain-compiler",
                "bijux-dna-domain-fastq",
                "bijux-dna-domain-vcf",
            ]
        );
        assert_eq!(CRATE_CYCLE_CLI_CRATES, &["bijux-dna", "bijux-dna-dev"]);
    }

    #[test]
    fn benchmarking_ready_gate_report_collects_passed_and_failed_goal_ids() {
        let report = build_benchmarking_ready_crate_shape_gate_report(
            Path::new("/workspace"),
            Path::new(
                "/workspace/artifacts/benchmark-smoke/readiness/crates/CRATE_SHAPE_FOR_BENCHMARKING_READY.json",
            ),
            Path::new("/workspace/artifacts/rust/crate-shape-gate-target"),
            vec![
                CrateShapeGateCheckReport {
                    goal_id: "411".to_string(),
                    ok: true,
                    command: "graph".to_string(),
                    detail: "ok".to_string(),
                },
                CrateShapeGateCheckReport {
                    goal_id: "416".to_string(),
                    ok: false,
                    command: "typed-proof".to_string(),
                    detail: "failed".to_string(),
                },
            ],
        );

        assert!(!report.ok);
        assert_eq!(report.passed_goal_ids, vec!["411".to_string()]);
        assert_eq!(report.failed_goal_ids, vec!["416".to_string()]);
        assert_eq!(
            report.output_path,
            "artifacts/benchmark-smoke/readiness/crates/CRATE_SHAPE_FOR_BENCHMARKING_READY.json"
        );
        assert_eq!(report.cargo_target_dir, "artifacts/rust/crate-shape-gate-target");
    }

    #[test]
    fn benchmarking_ready_gate_target_dir_defaults_to_artifacts_root() {
        let resolved = gate_cargo_target_dir_with_explicit_path(Path::new("/workspace"), None);

        assert_eq!(
            resolved,
            Path::new("/workspace").join("artifacts/rust/crate-shape-gate-target")
        );
    }

    #[test]
    fn benchmarking_ready_gate_target_dir_respects_explicit_path() {
        let resolved = gate_cargo_target_dir_with_explicit_path(
            Path::new("/workspace"),
            Some(Path::new("/tmp/custom-target")),
        );

        assert_eq!(resolved, Path::new("/tmp/custom-target"));
    }

    fn private_domain_audit_workspace(source: &str) -> anyhow::Result<tempfile::TempDir> {
        let root = tempfile::tempdir()?;
        let crate_root = root.path().join("crates/bijux-dna-domain-fastq");
        std::fs::create_dir_all(crate_root.join("src"))?;
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/bijux-dna-domain-fastq\"]\nresolver = \"2\"\n",
        )?;
        std::fs::write(
            crate_root.join("Cargo.toml"),
            "[package]\nname = \"bijux-dna-domain-fastq\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        std::fs::write(crate_root.join("src/lib.rs"), source)?;
        Ok(root)
    }

    #[test]
    fn crate_gate_refuses_persisted_failed_domain_audit() -> anyhow::Result<()> {
        let source = "pub fn forbidden_effect() { let _ = std::process::Command::new(\"sh\"); }\n";
        let root = private_domain_audit_workspace(source)?;
        let output = root.path().join("artifacts/domain-no-execution.json");
        let audit = super::write_domain_no_execution_report(root.path(), &output)?;
        assert_eq!(audit.audited_crate_count, 1);
        assert_eq!(audit.crates[0].crate_name, "bijux-dna-domain-fastq");
        assert!(!audit.ok, "the real writer must expose the admitted source violation");
        assert!(!audit.crates[0].process_execution_refs.is_empty());
        let persisted_before = std::fs::read(&output)?;
        let persisted: serde_json::Value = serde_json::from_slice(&persisted_before)?;
        assert_eq!(persisted["ok"], false);
        let expected_detail = audit.output_path.clone();
        let command = "domain-no-execution".to_string();
        let check = super::crate_shape_gate_check_from_result(
            "412",
            command.clone(),
            Ok(audit),
            |report| (report.ok, report.output_path),
        );
        assert!(!check.ok, "an Ok audit with ok=false must fail the crate gate");
        assert_eq!(check.goal_id, "412");
        assert_eq!(check.command, command);
        assert_eq!(check.detail, expected_detail);
        let gate = super::build_benchmarking_ready_crate_shape_gate_report(
            root.path(),
            &root.path().join("artifacts/gate.json"),
            &root.path().join("artifacts/gate-target"),
            vec![check],
        );
        assert!(!gate.ok);
        assert!(gate.passed_goal_ids.is_empty());
        assert_eq!(gate.failed_goal_ids, ["412"]);
        assert_eq!(std::fs::read(&output)?, persisted_before, "failed audit remains published");
        assert_eq!(
            std::fs::read_to_string(root.path().join("crates/bijux-dna-domain-fastq/src/lib.rs"))?,
            source
        );
        Ok(())
    }

    #[test]
    fn crate_gate_accepts_persisted_clean_domain_audit() -> anyhow::Result<()> {
        let root = private_domain_audit_workspace("pub fn pure_value() -> u64 { 7 }\n")?;
        let output = root.path().join("artifacts/domain-no-execution.json");
        let audit = super::write_domain_no_execution_report(root.path(), &output)?;
        assert_eq!(audit.audited_crate_count, 1);
        assert!(audit.ok);
        let expected_detail = audit.output_path.clone();
        let check = super::crate_shape_gate_check_from_result(
            "412",
            "domain-no-execution".to_string(),
            Ok(audit),
            |report| (report.ok, report.output_path),
        );
        assert!(check.ok);
        assert_eq!(check.detail, expected_detail);
        let gate = super::build_benchmarking_ready_crate_shape_gate_report(
            root.path(),
            &root.path().join("artifacts/gate.json"),
            &root.path().join("artifacts/gate-target"),
            vec![check],
        );
        assert!(gate.ok);
        assert_eq!(gate.passed_goal_ids, ["412"]);
        assert!(gate.failed_goal_ids.is_empty());
        let persisted: serde_json::Value = serde_json::from_slice(&std::fs::read(output)?)?;
        assert_eq!(persisted["ok"], true);
        Ok(())
    }

    #[test]
    fn crate_gate_refuses_domain_audit_acquisition_error() -> anyhow::Result<()> {
        let root = private_domain_audit_workspace("pub fn pure_value() -> u64 { 7 }\n")?;
        std::fs::write(root.path().join("Cargo.toml"), "[workspace\n")?;
        let output = root.path().join("artifacts/domain-no-execution.json");
        let result = super::write_domain_no_execution_report(root.path(), &output);
        let expected_error = match &result {
            Err(error) => error.to_string(),
            Ok(_) => anyhow::bail!("malformed manifest unexpectedly produced an audit"),
        };
        let check = super::crate_shape_gate_check_from_result(
            "412",
            "domain-no-execution".to_string(),
            result,
            |report| (report.ok, report.output_path),
        );
        assert!(!check.ok);
        assert_eq!(check.detail, expected_error);
        assert!(!output.exists(), "failed acquisition does not publish a fabricated audit");
        let gate = super::build_benchmarking_ready_crate_shape_gate_report(
            root.path(),
            &root.path().join("artifacts/gate.json"),
            &root.path().join("artifacts/gate-target"),
            vec![check],
        );
        assert!(!gate.ok);
        assert_eq!(gate.failed_goal_ids, ["412"]);
        Ok(())
    }
}
