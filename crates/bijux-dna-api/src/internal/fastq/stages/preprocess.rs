use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::support::benchmark_runtime::ensure_bench_runner;
use crate::support::workspace::load_workspace_registry;
use crate::tool_selection::filter_tools_by_role;
use crate::{execution_kernel, execution_kernel::NetworkPolicy};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench_results_fastq::SqliteBenchResultsRepository;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::ContainerImageRefV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_planner_fastq::{
    apply_preprocess_policy, preprocess_decisions, resolve_preprocess_pipeline,
    select_preprocess_stage_tools, FastqPlanConfig, FastqPlanner, StageToolSelection,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_runtime::recording::run_artifacts_dir_for_out;
use bijux_dna_runtime::recording::write_telemetry_event;
use bijux_dna_runtime::{
    attrs_from_json, build_telemetry_adapter, TelemetryEventName, TelemetryEventV1,
};

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::summary::{
    render_run_summary, report_stage_step, write_run_manifest, write_scientific_provenance,
    StageExecutionSummary,
};
use crate::internal::handlers::fastq::write_explain_plan_json;
use crate::internal::handlers::fastq::{
    STAGE_PREPROCESS_SUMMARY, STAGE_REPORT_QC, STAGE_TRIM_READS,
};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
};
use std::io::BufRead;
use std::path::PathBuf;

mod amplicon_governance;
mod amplicon_runtime;
mod coverage_regime;
mod invariants;
mod runtime_tail;
mod stage_artifacts;
mod stage_backend_policy;
mod stage_contracts;

pub(crate) use self::amplicon_governance::{resolve_primer_set_governance, PrimerSetGovernance};
pub use self::runtime_tail::{bench_fastq_preprocess, fastq_preprocess_run};

use self::amplicon_governance::{
    enforce_amplicon_merge_determinism, enforce_primer_governance, write_batch_effect_summary,
    write_contamination_controls_report, write_edna_report_summary,
    write_reference_db_validation_artifact,
};
use self::amplicon_runtime::{enforce_amplicon_qc_thresholds, materialize_amplicon_stage_outputs};
use self::coverage_regime::maybe_write_fastq_coverage_classifier;
use self::invariants::{open_fastq_lines, write_fastq_entry_invariants, FastqInvariantsReport};
use self::stage_artifacts::{emit_fastq_stage_extra_artifacts, write_stage_standardized_metrics};
use self::stage_backend_policy::{
    canonical_sample_identity, classify_failure_hint, enforce_fastq_backend_allowlist,
    enforce_metrics_schema, enforce_screen_db_governance, parse_first_u64_after_key,
    required_fastq_tools, stage_network_policy, write_retention_report, write_retry_policy,
};
use self::stage_contracts::{capture_tool_version, write_stage_path_contract};

use std::io::Read;

pub(crate) fn materialize_amplicon_stage_outputs_for_bench(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
) -> Result<serde_json::Value> {
    materialize_amplicon_stage_outputs(stage_root, planned)
}

pub(crate) fn enforce_amplicon_qc_thresholds_for_bench(
    stage_root: &std::path::Path,
    stage_id: &str,
    metrics: &serde_json::Value,
) -> Result<()> {
    enforce_amplicon_qc_thresholds(stage_root, stage_id, metrics)
}
