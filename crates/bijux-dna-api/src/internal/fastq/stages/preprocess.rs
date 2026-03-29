use bijux_dna_runtime::{
    attrs_from_json, build_telemetry_adapter, TelemetryEventName, TelemetryEventV1,
};
use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
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

pub(crate) use self::amplicon_governance::resolve_primer_set_governance;
pub use self::runtime_tail::{bench_fastq_preprocess, fastq_preprocess_run};

use self::amplicon_governance::*;
use self::amplicon_runtime::*;
use self::coverage_regime::*;
use self::invariants::*;
use self::runtime_tail::*;
use self::stage_artifacts::*;
use self::stage_backend_policy::*;

fn write_stage_path_contract(
    stage_root: &std::path::Path,
    stage_id: &str,
    planned: &ExecutionStep,
    is_paired: bool,
) -> Result<()> {
    bijux_dna_infra::ensure_dir(stage_root).context("create stage root for path contract")?;
    let outputs = planned
        .io
        .outputs
        .iter()
        .map(|x| {
            serde_json::json!({
                "name": x.name,
                "role": x.role.as_str(),
                "path": x.path
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.path_contract.v1",
        "stage_id": stage_id,
        "layout": if is_paired { "pe" } else { "se" },
        "deterministic_root": stage_root,
        "intermediate_root": stage_root.join("tmp"),
        "intermediate_paths": {
            "stdout_log": stage_root.join("stdout.log"),
            "stderr_log": stage_root.join("stderr.log"),
            "runtime_provenance": stage_root.join("runtime_provenance.json"),
            "resume_contract": stage_root.join("stage.resume_contract.json"),
        },
        "outputs": outputs,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.path_contract.json"), &payload)
        .context("write stage.path_contract.json")
}

fn capture_tool_version(stage_root: &std::path::Path, tool_bin: Option<&str>) -> Result<()> {
    let (declared_tool, ok, raw) = if let Some(tool_bin) = tool_bin.filter(|value| !value.trim().is_empty()) {
        let args = vec!["--version".to_string()];
        let output = bijux_dna_runner::command_runner::run_command(tool_bin, &args);
        let (ok, raw) = match output {
            Ok(out) => {
                let raw = if out.stdout.is_empty() {
                    out.stderr
                } else {
                    out.stdout
                };
                (out.exit_code == 0, raw)
            }
            Err(err) => (false, format!("failed to execute --version: {err}")),
        };
        (tool_bin, ok, raw)
    } else {
        ("", false, "tool command not declared in execution template".to_string())
    };
    let line = raw
        .lines()
        .find(|x| !x.trim().is_empty())
        .unwrap_or("")
        .trim();
    let tokenized = line
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '(' || c == ')')
        .filter(|x| !x.trim().is_empty())
        .collect::<Vec<_>>();
    let version = tokenized
        .iter()
        .find_map(|tok| {
            let t = tok.trim_start_matches('v').trim_start_matches('V');
            if t.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                Some(t.to_string())
            } else {
                None
            }
        });
    let payload = serde_json::json!({
        "schema_version": "bijux.tool_version_capture.v1",
        "tool": declared_tool,
        "ok": ok,
        "raw": raw,
        "parsed": {
            "first_line": line,
            "version": version
        }
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.tool_version.json"), &payload)
        .context("write stage.tool_version.json")
}

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
