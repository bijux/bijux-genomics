use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info_span, warn};

use crate::request_args::{
    ExecuteRunRequest, ExecuteRunResult, PlanRunRequest, PlanRunResult, RunRequest, RunResult,
};
use bijux_dna_core::contract::{Profile, RunSpec, ToolRegistry};
use bijux_dna_core::ids::RunId;
use bijux_dna_pipelines::Domain;
use bijux_dna_runtime::{ensure_stage_supported_by_runner, RunnerContractKind};
use bijux_dna_stage_contract::{build_run_execution_plan, RunExecutionPlan};

mod execution;
mod execution_support;
mod planning;
mod reporting;

use planning::{
    enforce_hpc_results_layout, file_len_i64, hpc_context_enabled, maybe_write_site_lock,
    millis_u64,
};

pub use execution::execute_run;
pub use planning::{
    explain_pipeline_profile, plan_only, plan_run, run_pipeline, select_pipeline,
    select_pipelines, stage_external_asset_requirement, stage_requires_local_assets,
    validate_pipeline_profile, RunMode, StageAssetClass, StageExternalAssetRequirement,
};
pub use reporting::{
    assess_failed_replay_eligibility, cancel_run, dry_run, environment_identity, execute,
    execute_and_report, execute_local_bam_workflow, execute_local_fastq_workflow,
    execute_local_vcf_workflow, explain_cache_hit_miss, explain_successful_replay, operator_health,
    pause_run, plan, policy_audit, render_report, replay_failed_run, replay_manifest, resume_run,
    status, workspace_edges, write_workspace_audit,
};
