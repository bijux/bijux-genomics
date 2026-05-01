//! Execution API for v1.
//!
//! Stability: v1 (stable).

mod entrypoints;
mod operator_failure;
mod request_contracts;
mod runtime_support;
mod stage_assets;

pub use entrypoints::{
    assess_failed_replay_eligibility, browse_runs, cache_explain, cancel_run, dry_run,
    environment_identity, evidence_gap, execute, execute_and_report, execute_local_bam_workflow,
    execute_local_fastq_workflow, execute_local_vcf_workflow, execute_run, explain_cache_hit_miss,
    explain_successful_replay, operator_diagnosis, operator_health, pause_run, plan, plan_only,
    policy_audit, query_run_lineage, render_operator_diagnosis_output, render_run_browser_output,
    replay_explain, replay_failed_run, replay_manifest, resume_run, run_fastq_to_bam_profile,
    run_local_failure_injection, run_pipeline, sign_bundle_prototype, status, verify_run_bundle,
    verify_signed_bundle_prototype, RunMode,
};
pub use operator_failure::{
    classify_operator_failure, CategorizedError, ErrorCategory, ErrorHintV1, HintSeverity,
    OperatorFailureV1,
};
pub use request_contracts::{
    CacheExplainRequestV1, CacheExplainResponseV1, CacheKeyFingerprintV1, CacheMissReasonV1,
    DryRunRequest, DryRunResponse, EvidenceCheckFailureV1, EvidenceGapRequestV1,
    EvidenceGapResponseV1, ExecuteRequest, ExecuteResponse, ExecuteRunRequest, ExecuteRunResult,
    OperatorDiagnosisCommandV1, OperatorDiagnosisRequestV1, OperatorDiagnosisResponseV1,
    OperatorHealthResponse, OutputFormatV1, PlanRequest, PlanResponse, RedactionProfileV1,
    ReplayExplainRequestV1, ReplayExplainResponseV1, RunBrowserFilterV1, RunBrowserRequestV1,
    RunBrowserResponseV1, RunBrowserRowV1, RunControlResponse, RunLineageEdgeV1,
    RunLineageQueryRequestV1, RunLineageQueryResponseV1, RunRequest, RunResult, RunStatus,
    SignedBundleRequestV1, SignedBundleResponseV1, SignedBundleVerifyRequestV1,
    SignedBundleVerifyResponseV1,
};
pub use runtime_support::{
    atomic_write_bytes, build_tool_execution_spec, ensure_dir, execution_step_from_stage_plan,
    init_logging, load_image_catalog, load_manifests, load_platform, load_profile, new_run_id,
    replay_run, resolve_run_base_dir, run_command, run_command_with_context, run_dir, temp_dir,
    temp_dir_in, write_bytes, write_plan_support_artifacts, CommandOutputV1, DryRunExecutor,
    ExecutionManifest, Executor, FactsRowV1, PathSpec, Profile, RunSpec, RuntimeKind, StageId,
    StagePlanV1, ToolId, ToolRegistry, ToolRole, RUN_LAYOUT_CONTRACT,
};
pub use stage_assets::{
    stage_external_asset_requirement, stage_requires_local_assets, StageAssetClass,
    StageExternalAssetRequirement,
};
