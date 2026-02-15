//! Execution API for v1.
//!
//! Stability: v1 (stable).

pub use crate::api_internal::handlers::cross::run_fastq_to_bam_profile;
pub use crate::request_args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, ExecuteRunRequest,
    ExecuteRunResult, PlanRequest, PlanResponse, RunRequest, RunResult, RunStatus,
};
pub use crate::run::{
    dry_run, execute, execute_and_report, execute_run, plan, plan_only, policy_audit,
    replay_manifest, run_pipeline, status, RunMode,
};
pub use bijux_dna_environment::api::{load_image_catalog, load_platform, RuntimeKind};
pub use bijux_dna_infra::RUN_LAYOUT_CONTRACT;
pub use bijux_dna_infra::{atomic_write_bytes, ensure_dir, temp_dir, temp_dir_in, write_bytes};

pub use bijux_dna_core::contract::*;
pub use bijux_dna_core::prelude::{
    run_dir, PathSpec, Profile, RunSpec, StageId, ToolId, ToolRegistry, ToolRole,
};
pub use bijux_dna_runtime::manifests::load_manifests;
pub use bijux_dna_runtime::run::{load_profile, new_run_id, resolve_run_base_dir};
pub use bijux_dna_runtime::FactsRowV1;
pub use bijux_dna_stage_contract::StagePlanV1;
pub use bijux_dna_stage_contract::{execution_step_from_stage_plan, DryRunExecutor, Executor};

pub use bijux_dna_core::contract::ExecutionManifest;
pub use bijux_dna_core::prelude::{CategorizedError, ErrorCategory};
pub use bijux_dna_core::prelude::{ErrorHintV1, HintSeverity};
pub use bijux_dna_infra::init_logging;
pub use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
pub use bijux_dna_runner::backend::docker::replay::replay_run;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// Stable operator-facing failure envelope surfaced by CLI/API.
///
/// Stability: v1 (stable).
pub struct OperatorFailureV1 {
    pub schema_version: String,
    pub category: ErrorCategory,
    pub message: String,
    pub hints: Vec<ErrorHintV1>,
}

#[must_use]
pub fn classify_operator_failure(err: &anyhow::Error) -> OperatorFailureV1 {
    let category = if let Some(categorized) = err.downcast_ref::<CategorizedError>() {
        categorized.category
    } else {
        err.chain()
            .find_map(|cause| {
                cause
                    .downcast_ref::<CategorizedError>()
                    .map(|categorized| categorized.category)
            })
            .unwrap_or(ErrorCategory::InfraError)
    };
    let hints = default_hints_for_category(category);
    OperatorFailureV1 {
        schema_version: "bijux.operator_failure.v1".to_string(),
        category,
        message: err.to_string(),
        hints,
    }
}

fn default_hints_for_category(category: ErrorCategory) -> Vec<ErrorHintV1> {
    let (id, message, action, docs_link_key) = match category {
        ErrorCategory::PlanError => (
            "plan.inputs",
            "input/run configuration is invalid or incomplete",
            "check required args, profile selection, and input file paths",
            Some("docs.plan_inputs".to_string()),
        ),
        ErrorCategory::ContractError => (
            "contract.violation",
            "a contract validation failed",
            "re-run with --dry-run and inspect manifest/contract diagnostics",
            Some("docs.contracts".to_string()),
        ),
        ErrorCategory::ParseError => (
            "observer.parse",
            "tool output could not be parsed by observer contract",
            "inspect stage logs and compare output format against fixture contracts",
            Some("docs.observer_parsers".to_string()),
        ),
        ErrorCategory::ToolError => (
            "tool.exit",
            "a tool invocation failed",
            "inspect tool stderr/stdout artifacts and adjust tool params or resources",
            Some("docs.tool_failures".to_string()),
        ),
        ErrorCategory::InfraError => (
            "infra.runtime",
            "runtime/environment failure during execution",
            "verify runner availability, image catalog, and filesystem permissions",
            Some("docs.runtime_failures".to_string()),
        ),
    };
    vec![ErrorHintV1 {
        id: id.to_string(),
        category,
        severity: HintSeverity::High,
        message: message.to_string(),
        suggested_action: action.to_string(),
        docs_link_key,
    }]
}
