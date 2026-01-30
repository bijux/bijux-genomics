use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::adapter_bank::{
    adapter_bank_provenance_json, resolve_adapter_selection, resolve_effective_adapters,
};
use anyhow::{anyhow, Context, Result};
use bijux_analyze::BenchmarkRecord;
use bijux_core::{CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1, ToolId};
use bijux_engine::api::StageResultV1;
use bijux_engine::api::{
    bench_base_dir, bench_tools_dir, execute_plan, PlatformSpec, RunnerKind, ToolImageSpec,
};
use bijux_engine::api::{ensure_bench_runner, filter_tools_by_role, load_registry};
use bijux_engine::api::{ensure_image_qa_passed, ensure_tool_qa_passed};
use bijux_engine::api::{hash_file_sha256, resolve_image_for_run, ExplainExclusion, ExplainPlan};
use bijux_stages_fastq::fastq::correct::{normalize_correct_tool_list, plan_correct};
use bijux_stages_fastq::fastq::filter::{normalize_filter_tool_list, plan_filter};
use bijux_stages_fastq::fastq::merge::{normalize_merge_tool_list, plan_merge};
use bijux_stages_fastq::fastq::preprocess::{plan_preprocess, plan_preprocess_pipeline};
use bijux_stages_fastq::fastq::qc_post::{aux_tool_ids, normalize_qc_post_tool_list, plan_qc_post};
use bijux_stages_fastq::fastq::screen::{normalize_screen_tool_list, plan_screen};
use bijux_stages_fastq::fastq::trim::plan;
use bijux_stages_fastq::fastq::umi::{normalize_umi_tool_list, plan_umi};
use bijux_stages_fastq::fastq::validate_pre::{
    normalize_validate_tool_list, plan as plan_validate_pre,
};
use bijux_stages_fastq::{
    bench_corpus, canonical_tool_defaults, ensure_umi_headers, inspect_headers,
    log_header_warnings, preflight_stage, FastqArtifactKind, RawFailure,
};
use bijux_stages_fastq::{FastqArtifact, FastqLayout};

pub struct BenchOutcome<M: bijux_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: std::path::PathBuf,
    pub explain: bool,
}

fn build_tool_execution_spec<S: ::std::hash::BuildHasher>(
    stage_id: &str,
    tool_id: &str,
    registry: &bijux_core::ToolRegistry,
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) -> Result<ToolExecutionSpecV1> {
    let manifest = registry
        .tool_by_id(stage_id, tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from manifest for {stage_id}"))?;
    let spec = catalog
        .get(tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    Ok(ToolExecutionSpecV1 {
        tool_id: ToolId(tool_id.to_string()),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: spec.digest.clone(),
        },
        command: CommandSpecV1 {
            template: manifest.command_template.clone(),
        },
        resources: manifest.constraints.clone(),
    })
}

fn adapter_bank_context(
    adapter_bank_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
    enable: &[String],
    disable: &[String],
) -> Result<Option<serde_json::Value>> {
    let selection =
        resolve_adapter_selection(adapter_bank_preset, legacy_adapter_bank, adapter_bank_file)?;
    let effective = resolve_effective_adapters(&selection, enable, disable)?;
    Ok(Some(adapter_bank_provenance_json(
        &selection, &effective, enable, disable,
    )))
}

struct StageExecutionSummary {
    plan: bijux_core::StagePlanV1,
    result: StageResultV1,
}

fn read_json_if_exists(path: &Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn write_run_summary(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[RawFailure],
) -> Result<()> {
    let root = out_dir.join("run_artifacts");
    fs::create_dir_all(&root).context("create run summary artifacts dir")?;
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir = entry.plan.out_dir.join("run_artifacts");
            let metrics_path = artifacts_dir.join("metrics_envelope.json");
            let metrics =
                read_json_if_exists(&metrics_path).and_then(|value| value.get("metrics").cloned());
            let stage_report_path = artifacts_dir.join("stage_report.json");
            let retention_report_path = artifacts_dir
                .join("reports")
                .join(format!("{}.retention.json", entry.plan.stage_id.0));
            serde_json::json!({
                "stage_id": entry.plan.stage_id.0,
                "tool_id": entry.plan.tool_id.0,
                "exit_code": entry.result.exit_code,
                "runtime_s": entry.result.runtime_s,
                "memory_mb": entry.result.memory_mb,
                "out_dir": entry.plan.out_dir,
                "artifacts": {
                    "metrics_envelope": metrics_path,
                    "stage_report": stage_report_path,
                    "retention_report": retention_report_path
                },
                "metrics": metrics.unwrap_or(serde_json::Value::Null)
            })
        })
        .collect();
    let failures_json: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason
            })
        })
        .collect();
    let summary = serde_json::json!({
        "schema_version": "bijux.run_summary.v1",
        "run_id": run_id,
        "stages": stages,
        "failures": failures_json
    });
    let summary_path = root.join("run_summary.json");
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary)?)
        .context("write run_summary.json")?;
    let html_path = root.join("run_summary.html");
    let html = render_run_summary_html(&summary);
    fs::write(&html_path, html).context("write run_summary.html")?;
    write_run_manifest(out_dir, stage_runs, failures)?;
    Ok(())
}

fn write_run_manifest(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[RawFailure],
) -> Result<()> {
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir = entry.plan.out_dir.join("run_artifacts");
            let mut artifacts = Vec::new();
            let add_artifact = |artifacts: &mut Vec<serde_json::Value>, name: &str, path: &Path| {
                if path.exists() {
                    if let Ok(hash) = hash_file_sha256(path) {
                        artifacts.push(serde_json::json!({
                            "name": name,
                            "path": path,
                            "sha256": hash,
                        }));
                    }
                }
            };
            add_artifact(
                &mut artifacts,
                "metrics_envelope",
                &artifacts_dir.join("metrics_envelope.json"),
            );
            add_artifact(
                &mut artifacts,
                "metrics",
                &artifacts_dir.join("metrics.json"),
            );
            add_artifact(
                &mut artifacts,
                "stage_metrics",
                &artifacts_dir.join("stage_metrics.json"),
            );
            add_artifact(
                &mut artifacts,
                "stage_report",
                &artifacts_dir.join("stage_report.json"),
            );
            add_artifact(
                &mut artifacts,
                "effective_config",
                &artifacts_dir.join("effective_config.json"),
            );
            add_artifact(
                &mut artifacts,
                "retention_report",
                &artifacts_dir
                    .join("reports")
                    .join(format!("{}.retention.json", entry.plan.stage_id.0)),
            );
            serde_json::json!({
                "stage_id": entry.plan.stage_id.0,
                "tool_id": entry.plan.tool_id.0,
                "artifacts": artifacts,
            })
        })
        .collect();
    let failures_json: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason
            })
        })
        .collect();
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v1",
        "run_id": run_id,
        "stages": stages,
        "failures": failures_json
    });
    let path = out_dir.join("run_manifest.json");
    fs::write(&path, serde_json::to_vec_pretty(&manifest)?).context("write run_manifest.json")?;
    Ok(())
}

fn render_run_summary_html(summary: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(summary).unwrap_or_else(|_| "{}".to_string());
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Bijux Run Summary</title>
  <style>
    body {{ font-family: 'Georgia', serif; margin: 2rem; background: #f7f3ef; color: #2b2b2b; }}
    h1 {{ font-size: 1.8rem; margin-bottom: 1rem; }}
    pre {{ background: #ffffff; padding: 1rem; border-radius: 8px; overflow-x: auto; }}
  </style>
</head>
<body>
  <h1>Run Summary</h1>
  <pre>{pretty}</pre>
</body>
</html>"#
    )
}

/// # Errors
/// Returns an error if the explain markdown cannot be written.
pub(crate) fn write_explain_md(
    base_dir: &Path,
    stage: &str,
    selected: &[String],
    excluded: &[String],
    policy: Option<bijux_engine::api::Policy>,
) -> Result<()> {
    let path = base_dir.join("explain.md");
    let mut lines = Vec::new();
    lines.push(format!("# Explain: {stage}"));
    if let Some(policy) = policy {
        lines.push(format!("\nPolicy: `{policy:?}`"));
    }
    lines.push("\n## Selected tools".to_string());
    for tool in selected {
        lines.push(format!("- {tool}"));
    }
    if !excluded.is_empty() {
        lines.push("\n## Excluded tools".to_string());
        for tool in excluded {
            lines.push(format!("- {tool}"));
        }
    }
    std::fs::write(&path, lines.join("\n")).context("write explain.md")?;
    Ok(())
}

/// # Errors
/// Returns an error if the explain plan JSON cannot be written.
pub(crate) fn write_explain_plan_json(
    base_dir: &Path,
    stage: &str,
    selected: &[String],
    registry: &bijux_core::ToolRegistry,
    policy: Option<bijux_engine::api::Policy>,
) -> Result<()> {
    let mut excluded = Vec::new();
    for tool in registry.tools_for_stage(stage) {
        if !selected.iter().any(|t| t == &tool.tool_id) {
            excluded.push(ExplainExclusion {
                tool: tool.tool_id.clone(),
                reason: "not selected".to_string(),
            });
        }
    }
    let invariants = vec![
        "stage_contract".to_string(),
        "header_inspection".to_string(),
        "output_normalization".to_string(),
    ];
    let plan = ExplainPlan {
        stage: stage.to_string(),
        selected_tools: selected.to_vec(),
        excluded_tools: excluded,
        policy,
        invariants,
    };
    let path = base_dir.join("explain_plan.json");
    bijux_engine::api::write_explain_plan(&path, &plan)
}

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> bijux_core::domain::PipelineSpec {
    plan_preprocess(args).pipeline
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_trim<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqTrimArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqTrimMetrics>> {
    let tools = bijux_engine::api::normalize_trim_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.trim", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.trim", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.trim", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "trim", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "trim", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.trim", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.trim", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.trim", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.trim", &tools, platform, catalog)?;

    let adapter_bank = adapter_bank_context(
        args.adapter_bank_preset.as_deref(),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.trim", tool, &registry, catalog, platform)?;
        let plan = plan(&tool_spec, &args.r1, &out_dir, adapter_bank.as_ref())?;
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.trim".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_validate_pre<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqValidateArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqValidateMetrics>> {
    let tools = normalize_validate_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.validate_pre", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.validate_pre", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.validate_pre", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "validate_pre", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "validate_pre", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.validate_pre", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.validate_pre", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.validate_pre", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.validate_pre", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.validate_pre", tool, &registry, catalog, platform)?;
        let plan = plan_validate_pre(&tool_spec, &args.r1, &out_dir);
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.validate_pre".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_filter<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqFilterArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqFilterMetrics>> {
    let tools = normalize_filter_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.filter", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.filter", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.filter", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "filter", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "filter", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.filter", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.filter", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.filter", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.filter", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.filter", tool, &registry, catalog, platform)?;
        let plan = plan_filter(&tool_spec, &args.r1, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.filter".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_merge<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqMergeArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqMergeMetrics>> {
    let tools = normalize_merge_tool_list(&args.tools)?;
    preflight_stage("fastq.merge", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, Some(&args.r2), false)?;
    log_header_warnings("fastq.merge", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.merge", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "merge", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "merge", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.merge", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.merge", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.merge", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.merge", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.merge", tool, &registry, catalog, platform)?;
        let plan = plan_merge(&tool_spec, &args.r1, &args.r2, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.merge".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_screen<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqScreenArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqScreenMetrics>> {
    let tools = normalize_screen_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.screen", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.screen", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.screen", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "screen", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "screen", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.screen", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.screen", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.screen", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.screen", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.screen", tool, &registry, catalog, platform)?;
        let plan = plan_screen(&tool_spec, &args.r1, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.screen".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_umi<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqUmiArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqUmiMetrics>> {
    let tools = normalize_umi_tool_list(&args.tools)?;
    let r2 = args
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("r2 required for fastq.umi"))?;
    preflight_stage("fastq.umi", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    ensure_umi_headers(&args.r1, args.r2.as_deref())?;
    log_header_warnings("fastq.umi", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.umi", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "umi", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "umi", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.umi", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.umi", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.umi", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.umi", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec = build_tool_execution_spec("fastq.umi", tool, &registry, catalog, platform)?;
        let plan = plan_umi(&tool_spec, &args.r1, r2, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.umi".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_correct<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqCorrectArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqCorrectMetrics>> {
    let tools = normalize_correct_tool_list(&args.tools)?;
    let r2 = args
        .r2
        .as_ref()
        .ok_or_else(|| anyhow!("r2 required for fastq.correct"))?;
    preflight_stage("fastq.correct", FastqArtifactKind::PairedEnd)?;
    let header = inspect_headers(&args.r1, args.r2.as_deref(), false)?;
    log_header_warnings("fastq.correct", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.correct", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "correct", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "correct", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.correct", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.correct", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.correct", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.correct", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.correct", tool, &registry, catalog, platform)?;
        let plan = plan_correct(&tool_spec, &args.r1, r2, &out_dir)?;
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.correct".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_qc_post<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqQcPostArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqQcPostMetrics>> {
    let tools = normalize_qc_post_tool_list(&args.tools)?;
    let artifact = FastqArtifact::single_end(&args.r1);
    preflight_stage("fastq.qc_post", artifact.kind)?;
    let header = inspect_headers(&args.r1, None, false)?;
    log_header_warnings("fastq.qc_post", &header);

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let tools = filter_tools_by_role("fastq.qc_post", &tools, &registry, false)?;

    let bench_dir = bench_base_dir(&args.out, "qc_post", &args.sample_id);
    let tools_root = bench_tools_dir(&args.out, "qc_post", &args.sample_id);
    fs::create_dir_all(&bench_dir).context("create bench output dir")?;
    fs::create_dir_all(&tools_root).context("create tools output dir")?;

    if args.explain {
        write_explain_md(&bench_dir, "fastq.qc_post", &tools, &[], None)?;
        write_explain_plan_json(&bench_dir, "fastq.qc_post", &tools, &registry, None)?;
    }

    ensure_bench_runner(platform, runner_override)?;
    ensure_image_qa_passed("fastq.qc_post", &tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.qc_post", &tools, platform, catalog)?;

    let mut failures = Vec::new();
    for tool in &tools {
        let out_dir = tools_root.join(tool);
        fs::create_dir_all(&out_dir).context("create tool output dir")?;
        let tool_spec =
            build_tool_execution_spec("fastq.qc_post", tool, &registry, catalog, platform)?;
        let mut aux_images = std::collections::BTreeMap::new();
        if tool == "multiqc" {
            for aux_tool in aux_tool_ids() {
                let spec = catalog
                    .get(*aux_tool)
                    .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.yaml"))?;
                let image = resolve_image_for_run(spec, platform)?;
                aux_images.insert(
                    (*aux_tool).to_string(),
                    ContainerImageRefV1 {
                        image: image.full_name,
                        digest: spec.digest.clone(),
                    },
                );
            }
        }
        let plan = plan_qc_post(&tool_spec, &args.r1, &out_dir, aux_images)?;
        let execution = execute_plan(&plan, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: "fastq.qc_post".to_string(),
                tool: tool.to_string(),
                reason: format!("tool {tool} failed with status {}", execution.exit_code),
            });
        }
    }

    Ok(BenchOutcome {
        records: Vec::new(),
        failures,
        bench_dir,
        explain: args.explain,
    })
}

/// Run the preprocess pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    fastq_preprocess_run(catalog, platform, runner_override, args)
}

/// Execute the preprocess pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;

    ensure_bench_runner(platform, runner_override)?;

    let registry = load_registry(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let pipeline = plan_preprocess(args).pipeline;
    let mut selected_tools = select_preprocess_tools(&registry, &pipeline, args)?;
    selected_tools = filter_tools_by_role("fastq.preprocess", &selected_tools, &registry, false)?;

    write_explain_plan_json(
        &out_dir,
        "fastq.preprocess",
        &selected_tools,
        &registry,
        None,
    )?;

    ensure_image_qa_passed("fastq.preprocess", &selected_tools, platform, catalog)?;
    ensure_tool_qa_passed("fastq.preprocess", &selected_tools, platform, catalog)?;

    let tools_root = bench_tools_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&tools_root).context("create preprocess tools dir")?;

    let mut failures = Vec::new();
    let mut tool_specs = Vec::new();
    for (stage, tool) in pipeline.stages.iter().zip(selected_tools.iter()) {
        tool_specs.push(build_tool_execution_spec(
            stage, tool, &registry, catalog, platform,
        )?);
    }
    let mut aux_tools = std::collections::BTreeMap::new();
    for aux_tool in bijux_stages_fastq::fastq::qc_post::aux_tool_ids() {
        let spec = catalog
            .get(*aux_tool)
            .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        aux_tools.insert(
            (*aux_tool).to_string(),
            ContainerImageRefV1 {
                image: image.full_name,
                digest: spec.digest.clone(),
            },
        );
    }
    let adapter_bank = adapter_bank_context(
        args.adapter_bank_preset.as_deref(),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let planned_stages = plan_preprocess_pipeline(
        &pipeline.stages,
        &tool_specs,
        &aux_tools,
        adapter_bank.as_ref(),
        &args.r1,
        args.r2.as_deref(),
        |stage, tool, _r1, _r2| {
            let stage_dir = stage.trim_start_matches("fastq.");
            let stage_root = bench_tools_dir(&args.out, stage_dir, &args.sample_id);
            let out_dir = stage_root.join(&tool.tool_id.0);
            fs::create_dir_all(&out_dir).context("create stage output dir")?;
            Ok(out_dir)
        },
    )?;

    let mut stage_runs = Vec::new();
    for planned in planned_stages {
        let stage_id = planned.stage_id.0.clone();
        let tool = planned.tool_id.0.clone();
        let execution = execute_plan(&planned, platform.runner)?;
        if execution.exit_code != 0 {
            failures.push(RawFailure {
                stage: stage_id,
                tool: tool.clone(),
                reason: format!("tool failed with status {}", execution.exit_code),
            });
        }
        stage_runs.push(StageExecutionSummary {
            plan: planned,
            result: execution,
        });
    }

    write_run_summary(&args.out, &stage_runs, &failures)?;
    if !failures.is_empty() {
        return Err(anyhow!(
            "preprocess pipeline failed: {} failures",
            failures.len()
        ));
    }

    let _ = if args.r2.is_some() {
        FastqLayout::PairedEnd
    } else {
        FastqLayout::SingleEnd
    };

    Ok(())
}

fn select_preprocess_tools(
    registry: &bijux_core::ToolRegistry,
    pipeline: &bijux_core::domain::PipelineSpec,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> Result<Vec<String>> {
    let defaults = canonical_tool_defaults();
    let mut selected_tools: Vec<String> = pipeline
        .stages
        .iter()
        .map(|stage| {
            defaults
                .get(stage.as_str())
                .map(|tool| (*tool).to_string())
                .or_else(|| {
                    registry
                        .tools_for_stage(stage)
                        .first()
                        .map(|tool| tool.tool_id.clone())
                })
                .ok_or_else(|| anyhow!("no default tool for stage {stage}"))
        })
        .collect::<Result<_>>()?;

    if args.auto {
        let corpus_id = args
            .bench_corpus
            .ok_or_else(|| anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bench_corpus(corpus_id);
        let objective = bijux_core::selection::objective_spec(args.objective);
        let mut selections = Vec::new();
        for stage in &pipeline.stages {
            let tool_ids: Vec<String> = registry
                .tools_for_stage(stage)
                .iter()
                .map(|tool| tool.tool_id.clone())
                .collect();
            let mut tool_records = Vec::new();
            for tool in &tool_ids {
                let records = bijux_stages_fastq::get_results(stage, tool, &corpus, &args.out)?;
                tool_records.push((tool.clone(), records));
            }
            let selection = bijux_core::selection::select_stage(
                stage,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            if selection.selected.is_none() {
                return Err(anyhow!(
                    "no eligible tools for {stage}; check bench corpus/results"
                ));
            }
            selections.push(selection);
        }
        selected_tools = selections
            .into_iter()
            .filter_map(|selection| selection.selected)
            .collect();
    }

    Ok(selected_tools)
}

/// Run the stats-neutral pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_stats_neutral<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &bijux_stages_fastq::args::BenchFastqStatsArgs,
) -> Result<BenchOutcome<bijux_analyze::FastqStatsMetrics>> {
    crate::fastq_stats_neutral::bench_fastq_stats_neutral(catalog, platform, runner_override, args)
}
