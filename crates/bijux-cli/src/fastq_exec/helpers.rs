use anyhow::{anyhow, Context, Result};
use bijux_engine::api::bench_tools_dir;
use bijux_engine::api::ResolvedImage;
use bijux_engine::api::{
    normalize_stats_tool_list as engine_normalize_stats_tool_list,
    normalize_trim_tool_list as engine_normalize_trim_tool_list,
    resolve_image_for_run as engine_resolve_image_for_run,
};
use bijux_environment::api::{PlatformSpec, ToolImageSpec};
use sha2::Digest;
use std::path::{Path, PathBuf};

use bijux_core::ToolRole;

use bijux_analyze::BenchmarkRecord;

use bijux_stages_fastq::{
    AdapterTrimmingReportV1, RawFailure, RetentionReportV1, StagePlanJson, ToolReferenceV1,
};

pub use bijux_engine::api::ExecutionManifest;
pub use bijux_engine::api::{ExplainExclusion, ExplainPlan};

#[derive(Debug)]
pub struct BenchOutcome<M: bijux_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: PathBuf,
    pub explain: bool,
}

#[derive(Debug)]
pub(crate) struct RunDirs {
    pub(crate) artifacts_dir: PathBuf,
    pub(crate) logs_dir: PathBuf,
    pub(crate) manifest_path: PathBuf,
    pub(crate) metrics_path: PathBuf,
    pub(crate) run_manifest_path: PathBuf,
    pub(crate) retention_report_path: PathBuf,
}

pub(crate) fn params_hash(params: &serde_json::Value) -> Result<String> {
    let bytes = serde_json::to_vec(params).context("serialize params")?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn compute_run_id(
    stage: &str,
    tool: &str,
    image_digest: &str,
    input_hash: &str,
    params_hash: &str,
) -> String {
    let seed = format!("{stage}|{tool}|{image_digest}|{input_hash}|{params_hash}");
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn prepare_tool_run_dirs(
    tools_root: &Path,
    tool: &str,
    run_id: &str,
) -> Result<RunDirs> {
    let adapter_bank_path = bijux_stages_fastq::adapter_bank_path();
    if !adapter_bank_path.exists() {
        return Err(anyhow!(
            "adapter bank missing at {}",
            adapter_bank_path.display()
        ));
    }
    let tool_dir = tools_root.join(tool);
    let run_dir = tool_dir.join("run").join(run_id);
    let artifacts_dir = run_dir.join("artifacts");
    let logs_dir = run_dir.join("logs");
    std::fs::create_dir_all(&artifacts_dir).context("create artifacts dir")?;
    std::fs::create_dir_all(&logs_dir).context("create logs dir")?;
    Ok(RunDirs {
        artifacts_dir,
        logs_dir: logs_dir.clone(),
        manifest_path: run_dir.join("manifest.json"),
        metrics_path: run_dir.join("metrics.json"),
        run_manifest_path: run_dir.join("run_manifest.json"),
        retention_report_path: run_dir.join("retention_report.json"),
    })
}

pub(crate) fn write_retention_report_placeholder(
    run_dirs: &RunDirs,
    stage: &str,
    tool: &str,
    params: &serde_json::Value,
) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": "bijux.retention_report.v1",
        "definition": "unknown/TBD",
        "numerator": "unknown/TBD",
        "denominator": "unknown/TBD",
        "scope": "unknown/TBD",
        "stage_boundary": format!("{stage}:unknown/TBD"),
        "tool": {
            "id": tool,
            "stage": stage,
            "version": "unknown/TBD",
            "params": params
        }
    });
    std::fs::write(
        &run_dirs.retention_report_path,
        serde_json::to_vec_pretty(&payload)?,
    )
    .context("write retention_report.json")?;
    Ok(())
}

pub(crate) fn write_run_manifest(
    run_dirs: &RunDirs,
    stage: &str,
    tool: &str,
    adapter_bank_path: &Path,
    extra_artifacts: &[RunArtifactInput],
) -> Result<()> {
    let mut artifacts = Vec::new();
    let has_retention_override = extra_artifacts
        .iter()
        .any(|artifact| artifact.name == "retention_report");
    let manifest_hash = bijux_engine::api::hash_file_sha256(&run_dirs.manifest_path)?;
    artifacts.push(serde_json::json!({
        "name": "execution_manifest",
        "path": run_dirs.manifest_path,
        "sha256": manifest_hash
    }));
    let metrics_hash = bijux_engine::api::hash_file_sha256(&run_dirs.metrics_path)?;
    artifacts.push(serde_json::json!({
        "name": "metrics",
        "path": run_dirs.metrics_path,
        "sha256": metrics_hash
    }));
    if !has_retention_override {
        let retention_hash = bijux_engine::api::hash_file_sha256(&run_dirs.retention_report_path)?;
        artifacts.push(serde_json::json!({
            "name": "retention_report",
            "path": run_dirs.retention_report_path,
            "sha256": retention_hash
        }));
    }
    let adapter_hash = bijux_engine::api::hash_file_sha256(adapter_bank_path)?;
    artifacts.push(serde_json::json!({
        "name": "adapter_bank",
        "path": adapter_bank_path,
        "sha256": adapter_hash
    }));
    for artifact in extra_artifacts {
        let hash = bijux_engine::api::hash_file_sha256(&artifact.path)?;
        artifacts.push(serde_json::json!({
            "name": artifact.name,
            "path": artifact.path,
            "sha256": hash
        }));
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.run_manifest.v1",
        "stage": stage,
        "tool": tool,
        "artifacts": artifacts
    });
    std::fs::write(
        &run_dirs.run_manifest_path,
        serde_json::to_vec_pretty(&payload)?,
    )
    .context("write run_manifest.json")?;
    Ok(())
}

fn run_artifacts_dir(run_dirs: &RunDirs) -> Result<PathBuf> {
    let run_dir = run_dirs
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run dir missing for manifest"))?;
    Ok(run_dir.join("run_artifacts"))
}

pub(crate) struct RunArtifactInput {
    pub(crate) name: &'static str,
    pub(crate) path: PathBuf,
}

pub(crate) fn write_effective_adapters(
    run_dirs: &RunDirs,
    effective: &bijux_stages_fastq::EffectiveAdapterSet,
    bank_checksum: &str,
    presets_checksum: &str,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dirs)?;
    let adapters_dir = root.join("adapters");
    std::fs::create_dir_all(&adapters_dir).context("create adapters artifact dir")?;
    let path = adapters_dir.join("effective_adapters.json");
    let adapters: Vec<serde_json::Value> = effective
        .adapters
        .iter()
        .map(|adapter| {
            serde_json::json!({
                "id": adapter.id,
                "sequence": adapter.sequence,
            })
        })
        .collect();
    let payload = serde_json::json!({
        "schema_version": "bijux.effective_adapters.v1",
        "preset": effective.preset,
        "enabled_adapter_ids": effective.enabled_ids,
        "adapters": adapters,
        "bank_checksum": bank_checksum,
        "presets_checksum": presets_checksum
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write effective_adapters.json")?;
    Ok(path)
}

pub(crate) fn write_adapter_bank_ref(
    run_dirs: &RunDirs,
    bank: &bijux_stages_fastq::AdapterBankV1,
    bank_path: &Path,
    presets_path: &Path,
    bank_checksum: &str,
    presets_checksum: &str,
    effective: &bijux_stages_fastq::EffectiveAdapterSet,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dirs)?;
    let adapters_dir = root.join("adapters");
    std::fs::create_dir_all(&adapters_dir).context("create adapters artifact dir")?;
    let path = adapters_dir.join("adapter_bank_ref.json");
    let payload = serde_json::json!({
        "schema_version": "bijux.adapter_bank_ref.v1",
        "bank_version": bank.schema_version,
        "bank_checksum": bank_checksum,
        "presets_checksum": presets_checksum,
        "preset": effective.preset,
        "enabled_adapter_ids": effective.enabled_ids,
        "sources": {
            "bank_path": bank_path.display().to_string(),
            "presets_path": presets_path.display().to_string()
        }
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write adapter_bank_ref.json")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_adapter_trimming_report(
    run_dirs: &RunDirs,
    tool: &str,
    tool_version: &str,
    params: &serde_json::Value,
    total_reads: u64,
    reads_with_adapter: u64,
    bases_trimmed_total: u64,
    per_adapter_counts: std::collections::BTreeMap<String, u64>,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dirs)?;
    let reports_dir = root.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports artifact dir")?;
    let path = reports_dir.join("adapter_trimming_report.json");
    let report = AdapterTrimmingReportV1 {
        schema_version: "bijux.adapter_trimming_report.v1".to_string(),
        reads_with_adapter,
        total_reads,
        bases_trimmed_total,
        per_adapter_counts,
        top_k_adapters: Vec::new(),
        tool: ToolReferenceV1 {
            id: tool.to_string(),
            stage: "fastq.trim".to_string(),
            version: tool_version.to_string(),
            params: params.clone(),
        },
    };
    std::fs::write(&path, serde_json::to_vec_pretty(&report)?)
        .context("write adapter_trimming_report.json")?;
    Ok(path)
}

pub(crate) fn write_retention_report_artifact(
    run_dirs: &RunDirs,
    report: &RetentionReportV1,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dirs)?;
    let reports_dir = root.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports artifact dir")?;
    let path = reports_dir.join("retention_report.json");
    std::fs::write(&path, serde_json::to_vec_pretty(report)?)
        .context("write retention_report.json")?;
    Ok(path)
}

pub(crate) fn write_stage_plan_json(
    run_dirs: &RunDirs,
    file_name: &str,
    plan: &StagePlanJson,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dirs)?;
    let plans_dir = root.join("plans");
    std::fs::create_dir_all(&plans_dir).context("create plans artifact dir")?;
    let path = plans_dir.join(file_name);
    std::fs::create_dir_all(path.parent().unwrap_or(&plans_dir))
        .context("create plan parent dir")?;
    std::fs::write(&path, serde_json::to_vec_pretty(plan)?).context("write stage plan json")?;
    Ok(path)
}

#[allow(dead_code)]
pub(crate) fn tool_run_artifacts_dir(
    out: &Path,
    stage: &str,
    sample_id: &str,
    tool: &str,
    run_id: &str,
) -> PathBuf {
    bench_tools_dir(out, stage, sample_id)
        .join(tool)
        .join("run")
        .join(run_id)
        .join("artifacts")
}

pub(crate) fn write_execution_logs(run_dirs: &RunDirs, stdout: &str, stderr: &str) -> Result<()> {
    let log_path = run_dirs.logs_dir.join("tool.log");
    if stderr.is_empty() {
        std::fs::write(&log_path, stdout).context("write tool.log")?;
    } else {
        std::fs::write(&log_path, format!("{stdout}\n--- stderr ---\n{stderr}"))
            .context("write tool.log")?;
    }
    Ok(())
}

pub(crate) fn write_metrics_json<T: serde::Serialize>(
    run_dirs: &RunDirs,
    execution: &bijux_core::measure::ExecutionMetrics,
    metrics: &bijux_core::metrics::MetricEnvelope<T>,
) -> Result<()> {
    let payload = serde_json::json!({
        "execution": execution,
        "metrics": metrics
    });
    std::fs::write(&run_dirs.metrics_path, serde_json::to_vec_pretty(&payload)?)
        .context("write metrics.json")?;
    Ok(())
}

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

pub(crate) fn normalize_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_trim_tool_list(tools)
}

pub(crate) fn normalize_stats_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_stats_tool_list(tools)
}

pub(crate) fn filter_tools_by_role(
    stage_id: &str,
    tools: &[String],
    registry: &bijux_core::ToolRegistry,
    strict: bool,
) -> Result<Vec<String>> {
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let mut filtered = Vec::new();
    for tool in tools {
        let manifest = registry
            .tool_by_id(stage_id, tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
        match manifest.role {
            ToolRole::Authoritative => filtered.push(tool.clone()),
            ToolRole::Diagnostic => {
                if strict {
                    return Err(anyhow!(
                        "strict mode requires authoritative tools; {tool} is diagnostic"
                    ));
                }
                filtered.push(tool.clone());
            }
            ToolRole::Experimental => {
                if !allow_experimental {
                    return Err(anyhow!(
                        "experimental tool {tool} requires BIJUX_EXPERIMENTAL_TOOLS=1"
                    ));
                }
                filtered.push(tool.clone());
            }
        }
    }
    if filtered.is_empty() {
        return Err(anyhow!("no tools available after role filtering"));
    }
    Ok(filtered)
}

pub(crate) fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    engine_resolve_image_for_run(spec, platform)
}
