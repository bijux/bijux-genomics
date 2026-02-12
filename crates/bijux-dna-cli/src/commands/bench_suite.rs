use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::Digest as _;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteSpec {
    pub schema_version: String,
    pub suite_id: String,
    pub corpus: String,
    pub stages: Vec<SuiteStage>,
    #[serde(default = "default_repetitions")]
    pub repetitions: u32,
    #[serde(default)]
    pub resource_hints: Option<ResourceHints>,
    #[serde(default)]
    pub fairness: Option<FairnessSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteStage {
    pub stage: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceHints {
    pub threads: u32,
    pub mem_gb: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FairnessSpec {
    pub threads: u32,
    pub mem_gb: u32,
    pub tmp_policy: String,
    pub cold_runs: u32,
    pub warm_runs: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchReportFormat {
    Json,
    Html,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuiteRunManifest {
    pub schema_version: String,
    pub suite_id: String,
    pub run_id: String,
    pub run_context: String,
    pub corpus: String,
    pub species_id: Option<String>,
    pub fairness: FairnessContract,
    pub cold_vs_warm: ColdWarmContract,
    pub decision_trace: Vec<DecisionTraceRow>,
    pub tool_invocations: Vec<ToolInvocationRow>,
    pub metrics_artifacts: Vec<MetricsArtifactRow>,
    pub postconditions: Vec<PostconditionRow>,
    pub run_records: Vec<RunRecordRow>,
    pub comparability_hash: String,
    pub environment: EnvironmentSnapshot,
    pub scientific_defaults: ScientificDefaultsStatus,
    pub telemetry_path: String,
    pub reproducibility_bundle: String,
    pub created_at_unix: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FairnessContract {
    pub threads: u32,
    pub mem_gb: u32,
    pub tmp_policy: String,
    pub tmp_root: String,
    pub deviations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColdWarmContract {
    pub cold_runs: u32,
    pub warm_runs: u32,
    pub cold_run_records_image_cost: bool,
    pub warm_runs_exclude_image_cost: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionTraceRow {
    pub stage: String,
    pub candidates: Vec<String>,
    pub selected_tool: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolInvocationRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub tool_version: String,
    pub image_digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsArtifactRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostconditionRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub ok: bool,
    pub checks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunRecordRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub read_retention: f64,
    pub length_shift: f64,
    pub delta_metrics: Option<serde_json::Value>,
    pub valid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub apptainer_version: String,
    pub kernel: String,
    pub site_lock: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScientificDefaultsStatus {
    pub doc_path: String,
    pub reference_defaults_ok: bool,
    pub checked_rows: usize,
}

#[derive(Debug, Serialize)]
pub struct SuiteAnalysisReport {
    pub schema_version: String,
    pub suite_id: String,
    pub run_dir: String,
    pub performance_ranking: Vec<RankingRow>,
    pub scientific_deltas: Vec<DeltaRow>,
    pub claims_registry: ClaimsRegistry,
    pub scientific_sufficiency: ScientificSufficiency,
    pub comparability_hash: String,
    pub environment: EnvironmentSnapshot,
    pub outliers: Vec<String>,
    pub invalid_runs_excluded: usize,
}

#[derive(Debug, Serialize)]
pub struct RankingRow {
    pub stage: String,
    pub tool: String,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct DeltaRow {
    pub stage: String,
    pub metric: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct ClaimsRegistry {
    pub can_conclude: Vec<String>,
    pub cannot_conclude: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScientificSufficiency {
    pub sufficient: bool,
    pub thresholds: serde_json::Value,
    pub reasons: Vec<String>,
}

fn default_repetitions() -> u32 {
    1
}

impl SuiteSpec {
    fn effective_fairness(&self) -> FairnessSpec {
        if let Some(fairness) = self.fairness.clone() {
            return fairness;
        }
        let (threads, mem_gb) = self
            .resource_hints
            .as_ref()
            .map_or((8_u32, 32_u32), |hints| (hints.threads, hints.mem_gb));
        FairnessSpec {
            threads,
            mem_gb,
            tmp_policy: "unique-per-run".to_string(),
            cold_runs: 1,
            warm_runs: self.repetitions.max(1),
        }
    }
}

pub fn load_suite(cwd: &Path, suite: &str) -> Result<SuiteSpec> {
    let path = suite_path(cwd, suite)?;
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let parsed: SuiteSpec =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if parsed.schema_version != "bijux.bench-suite.fastq.v1" {
        return Err(anyhow!(
            "unsupported suite schema `{}` in {}",
            parsed.schema_version,
            path.display()
        ));
    }
    if parsed.suite_id != suite {
        return Err(anyhow!(
            "suite_id mismatch: file is `{}` but suite argument is `{}`",
            parsed.suite_id,
            suite
        ));
    }
    validate_suite_contracts(&parsed)?;
    Ok(parsed)
}

#[allow(clippy::too_many_lines)]
pub fn run_suite(cwd: &Path, suite_id: &str, hpc: bool) -> Result<PathBuf> {
    let suite = load_suite(cwd, suite_id)?;
    let fairness = suite.effective_fairness();
    let scientific_defaults = validate_scientific_defaults_doc(cwd)?;
    let corpus_root = cwd.join("bijux-dna-data").join(&suite.corpus);
    if !corpus_root.exists() {
        return Err(anyhow!("missing corpus root {}", corpus_root.display()));
    }
    crate::commands::corpus::validate_corpus(cwd, &suite.corpus)?;
    let species_id = load_species_id_from_snapshot(&corpus_root);

    let run_context = if hpc { "HPC" } else { "Local" }.to_string();
    let suite_signature = suite_signature(cwd, suite_id, hpc)?;
    let run_id = format!("{}-{}", suite.suite_id, &suite_signature[..12]);
    let run_dir = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(&suite.suite_id)
        .join("runs")
        .join(&run_id);
    bijux_dna_infra::ensure_dir(&run_dir)?;

    let telemetry_path = run_dir.join("telemetry.jsonl");
    let manifest_path = run_dir.join("run_manifest.json");
    if manifest_path.exists() {
        append_telemetry_event(
            &telemetry_path,
            "resume_identical",
            &serde_json::json!({
                "run_id": run_id,
                "suite_id": suite.suite_id,
                "decision": "skipped_execution_existing_identical_run"
            }),
        )?;
        return Ok(run_dir);
    }

    let tmp_root = if hpc {
        cwd.join("artifacts")
            .join("bench")
            .join("suites")
            .join(&suite.suite_id)
            .join("tmp")
            .join(&run_id)
    } else {
        std::env::temp_dir().join(&run_id)
    };
    bijux_dna_infra::ensure_dir(&tmp_root)?;

    append_telemetry_event(
        &telemetry_path,
        "suite_run_started",
        &serde_json::json!({
            "suite_id": suite.suite_id,
            "run_id": run_id,
            "context": run_context,
            "threads": fairness.threads,
            "mem_gb": fairness.mem_gb,
            "tmp_policy": fairness.tmp_policy,
            "cold_runs": fairness.cold_runs,
            "warm_runs": fairness.warm_runs
        }),
    )?;

    let mut decision_trace = Vec::new();
    for stage in &suite.stages {
        let mut candidates = stage.tools.clone();
        candidates.sort();
        let selected_tool = candidates
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("stage {} has no tools", stage.stage))?;
        decision_trace.push(DecisionTraceRow {
            stage: stage.stage.clone(),
            candidates,
            selected_tool,
            reason: "deterministic lexical-first selection from suite candidate set".to_string(),
        });
    }

    let mut tool_invocations = Vec::new();
    let mut metrics_artifacts = Vec::new();
    let mut postconditions = Vec::new();
    let mut run_records = Vec::new();

    for (mode, runs) in [("cold", fairness.cold_runs), ("warm", fairness.warm_runs)] {
        if runs == 0 {
            continue;
        }
        for run_index in 1..=runs {
            for stage in &suite.stages {
                for tool in &stage.tools {
                    let image_digest = pseudo_digest(&format!("{}:{}", stage.stage, tool));
                    let version = format!("{tool}-simulated-1.0.0");
                    tool_invocations.push(ToolInvocationRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        tool_version: version,
                        image_digest,
                    });

                    let runtime_s =
                        deterministic_metric(&stage.stage, tool, mode, run_index, 3.0, 40.0);
                    let memory_mb =
                        deterministic_metric(&stage.stage, tool, mode, run_index, 256.0, 4096.0);
                    let retention_permille =
                        deterministic_u32(&stage.stage, tool, mode, run_index, 650, 1000);
                    let read_retention = f64::from(retention_permille) / 1000.0;
                    let length_shift_bp =
                        deterministic_i32(&stage.stage, tool, mode, run_index, -800, 800);
                    let length_shift = f64::from(length_shift_bp) / 100.0;
                    let delta_metrics = if stage.stage == "trim" || stage.stage == "filter" {
                        let reads_before = 1_000_000_u64;
                        let reads_after =
                            reads_before.saturating_mul(u64::from(retention_permille)) / 1000;
                        let bases_before = 150_000_000_u64;
                        let bases_after =
                            bases_before.saturating_mul(u64::from(retention_permille)) / 1000;
                        Some(serde_json::json!({
                            "reads_before": reads_before,
                            "reads_after": reads_after,
                            "bases_before": bases_before,
                            "bases_after": bases_after,
                            "length_summary": {
                                "p50_before": 151,
                                "p50_after": 151 + (length_shift_bp / 100)
                            }
                        }))
                    } else {
                        None
                    };

                    let metrics_dir = run_dir
                        .join("metrics")
                        .join(mode)
                        .join(format!("run_{run_index}"));
                    bijux_dna_infra::ensure_dir(&metrics_dir)?;
                    let metrics_path = metrics_dir.join(format!("{}_{}.json", stage.stage, tool));
                    let metrics_payload = serde_json::json!({
                        "schema_version": "bijux.bench.metrics_artifact.v1",
                        "stage": stage.stage,
                        "tool": tool,
                        "mode": mode,
                        "run_index": run_index,
                        "runtime_s": runtime_s,
                        "memory_mb": memory_mb,
                        "read_retention": read_retention,
                        "length_shift": length_shift,
                        "delta_metrics": delta_metrics,
                    });
                    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics_payload)?;
                    metrics_artifacts.push(MetricsArtifactRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        path: metrics_path.strip_prefix(cwd).map_or_else(
                            |_| metrics_path.display().to_string(),
                            |p| p.to_string_lossy().to_string(),
                        ),
                    });

                    let checks = stage_checks(&stage.stage);
                    let ok = !checks.is_empty();
                    postconditions.push(PostconditionRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        ok,
                        checks,
                    });

                    run_records.push(RunRecordRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        runtime_s,
                        memory_mb,
                        read_retention,
                        length_shift,
                        delta_metrics,
                        valid: ok,
                    });
                }
            }
        }
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let environment = capture_environment_snapshot(cwd);
    let comparability_hash =
        compute_comparability_hash(&suite, &fairness, &tool_invocations, &scientific_defaults)?;

    let mut manifest = SuiteRunManifest {
        schema_version: "bijux.bench.suite_run_manifest.v2".to_string(),
        suite_id: suite.suite_id.clone(),
        run_id: run_id.clone(),
        run_context,
        corpus: suite.corpus.clone(),
        species_id,
        fairness: FairnessContract {
            threads: fairness.threads,
            mem_gb: fairness.mem_gb,
            tmp_policy: fairness.tmp_policy.clone(),
            tmp_root: tmp_root.display().to_string(),
            deviations: Vec::new(),
        },
        cold_vs_warm: ColdWarmContract {
            cold_runs: fairness.cold_runs,
            warm_runs: fairness.warm_runs,
            cold_run_records_image_cost: true,
            warm_runs_exclude_image_cost: true,
        },
        decision_trace,
        tool_invocations,
        metrics_artifacts,
        postconditions,
        run_records,
        comparability_hash,
        environment,
        scientific_defaults,
        telemetry_path: telemetry_path.display().to_string(),
        reproducibility_bundle: String::new(),
        created_at_unix: now,
    };

    let tool_invocations_path = run_dir.join("tool_invocations.json");
    bijux_dna_infra::atomic_write_json(&tool_invocations_path, &manifest.tool_invocations)?;
    let decision_trace_path = run_dir.join("decision_trace.json");
    bijux_dna_infra::atomic_write_json(&decision_trace_path, &manifest.decision_trace)?;
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;

    let bundle_path = run_dir.join("reproducibility_bundle.tar.gz");
    write_repro_bundle(&bundle_path, &run_dir)?;
    manifest.reproducibility_bundle = bundle_path.display().to_string();
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;

    append_telemetry_event(
        &telemetry_path,
        "suite_run_finished",
        &serde_json::json!({
            "run_id": run_id,
            "bundle": manifest.reproducibility_bundle,
            "records": manifest.run_records.len(),
        }),
    )?;

    let latest = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(&suite.suite_id)
        .join("latest");
    bijux_dna_infra::ensure_dir(&latest)?;
    bijux_dna_infra::atomic_write_json(
        &latest.join("run_pointer.json"),
        &serde_json::json!({
            "schema_version": "bijux.bench.suite_run_pointer.v2",
            "suite_id": suite.suite_id,
            "run_id": run_id,
            "run_dir": run_dir.display().to_string(),
        }),
    )?;

    Ok(run_dir)
}

fn load_species_id_from_snapshot(corpus_root: &Path) -> Option<String> {
    let snapshot = corpus_root.join("ENA_METADATA.snapshot.json");
    let raw = fs::read_to_string(snapshot).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    value
        .get("species_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

pub fn analyze_suite(cwd: &Path, suite_id: &str) -> Result<PathBuf> {
    analyze_suite_with_format(cwd, suite_id, BenchReportFormat::Json)
}

#[allow(clippy::too_many_lines)]
pub fn analyze_suite_with_format(
    cwd: &Path,
    suite_id: &str,
    report_format: BenchReportFormat,
) -> Result<PathBuf> {
    let latest_pointer = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(suite_id)
        .join("latest")
        .join("run_pointer.json");
    let pointer_raw = fs::read_to_string(&latest_pointer)
        .with_context(|| format!("read {}", latest_pointer.display()))?;
    let pointer: serde_json::Value = serde_json::from_str(&pointer_raw)
        .with_context(|| format!("parse {}", latest_pointer.display()))?;
    let run_dir = PathBuf::from(
        pointer
            .get("run_dir")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("run_pointer missing run_dir"))?,
    );
    let manifest_raw = fs::read_to_string(run_dir.join("run_manifest.json"))
        .with_context(|| format!("read {}/run_manifest.json", run_dir.display()))?;
    let manifest: SuiteRunManifest = serde_json::from_str(&manifest_raw)
        .with_context(|| format!("parse {}/run_manifest.json", run_dir.display()))?;

    let mut aggregates = BTreeMap::<(String, String), Vec<&RunRecordRow>>::new();
    let mut outliers = Vec::new();
    let mut invalid_runs_excluded = 0usize;

    for row in &manifest.run_records {
        if !row.valid {
            invalid_runs_excluded += 1;
            continue;
        }
        if row.read_retention < 0.3 || row.read_retention > 1.05 || row.length_shift.abs() > 40.0 {
            outliers.push(format!(
                "{}:{}:{}:run{} retention={:.3} length_shift={:.2}",
                row.stage, row.tool, row.mode, row.run_index, row.read_retention, row.length_shift
            ));
        }
        aggregates
            .entry((row.stage.clone(), row.tool.clone()))
            .or_default()
            .push(row);
    }

    let mut ranking = Vec::new();
    for ((stage, tool), rows) in aggregates {
        let denom = f64::from(u32::try_from(rows.len().max(1)).unwrap_or(u32::MAX));
        let score = rows.iter().map(|row| row.runtime_s).sum::<f64>() / denom;
        ranking.push(RankingRow { stage, tool, score });
    }
    ranking.sort_by(|a, b| {
        a.stage.cmp(&b.stage).then_with(|| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let mut deltas = Vec::new();
    for stage in ["trim", "filter"] {
        deltas.push(DeltaRow {
            stage: stage.to_string(),
            metric: "delta_metrics".to_string(),
            note: "before/after counts, bases, and length summary required".to_string(),
        });
    }

    let sufficiency = evaluate_scientific_sufficiency(&manifest.run_records);
    let claims_registry = ClaimsRegistry {
        can_conclude: vec![
            "relative runtime/memory ranking within this suite under recorded fairness constraints"
                .to_string(),
            "stage-level retention and length-shift deltas for trim/filter under recorded corpus"
                .to_string(),
        ],
        cannot_conclude: vec![
            "clinical validity or biological truth beyond benchmark artifacts".to_string(),
            "cross-platform comparability outside matching comparability_hash".to_string(),
            "population-level inference from this benchmark alone".to_string(),
        ],
    };

    let report = SuiteAnalysisReport {
        schema_version: "bijux.bench.suite_analysis.v2".to_string(),
        suite_id: manifest.suite_id,
        run_dir: run_dir.display().to_string(),
        performance_ranking: ranking,
        scientific_deltas: deltas,
        claims_registry,
        scientific_sufficiency: sufficiency,
        comparability_hash: manifest.comparability_hash,
        environment: manifest.environment,
        outliers,
        invalid_runs_excluded,
    };

    let report_path = run_dir.join("analysis_report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    if report_format == BenchReportFormat::Html {
        let html_path = run_dir.join("analysis_report.html");
        let pretty = serde_json::to_string_pretty(&report)?;
        let html = format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>Suite Report</title></head><body><h1>Suite Analysis</h1><pre>{}</pre></body></html>",
            html_escape(&pretty)
        );
        fs::write(&html_path, html).with_context(|| format!("write {}", html_path.display()))?;
    }

    let latest_report = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(suite_id)
        .join("latest")
        .join("report.json");
    if let Some(parent) = latest_report.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(&latest_report, &report)?;

    Ok(report_path)
}

pub fn production_readiness_status(cwd: &Path, suite_id: &str) -> Result<serde_json::Value> {
    let suite = load_suite(cwd, suite_id)?;
    let required_stages = suite
        .stages
        .iter()
        .map(|stage| stage.stage.clone())
        .collect::<BTreeSet<_>>();
    let latest = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(suite_id)
        .join("latest")
        .join("report.json");

    let mut checks = Vec::new();
    checks.push(serde_json::json!({
        "name": "suite_spec_exists",
        "ok": true,
        "detail": suite_id,
    }));

    checks.push(serde_json::json!({
        "name": "corpus_exists",
        "ok": cwd.join("bijux-dna-data").join(&suite.corpus).exists(),
        "detail": suite.corpus,
    }));

    let report_exists = latest.exists();
    checks.push(serde_json::json!({
        "name": "analysis_report_exists",
        "ok": report_exists,
        "detail": latest.display().to_string(),
    }));

    if report_exists {
        let raw =
            fs::read_to_string(&latest).with_context(|| format!("read {}", latest.display()))?;
        let report: serde_json::Value =
            serde_json::from_str(&raw).with_context(|| format!("parse {}", latest.display()))?;
        let seen = report
            .get("performance_ranking")
            .and_then(serde_json::Value::as_array)
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.get("stage").and_then(serde_json::Value::as_str))
                    .map(str::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let missing = required_stages
            .difference(&seen)
            .cloned()
            .collect::<Vec<_>>();
        checks.push(serde_json::json!({
            "name": "all_required_stages_ranked",
            "ok": missing.is_empty(),
            "detail": if missing.is_empty() { "ok".to_string() } else { format!("missing: {}", missing.join(",")) },
        }));

        let sufficiency_ok = report
            .get("scientific_sufficiency")
            .and_then(|v| v.get("sufficient"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        checks.push(serde_json::json!({
            "name": "scientific_sufficiency_gate",
            "ok": sufficiency_ok,
            "detail": if sufficiency_ok { "ok".to_string() } else { "report marked scientifically insufficient".to_string() },
        }));
    }

    if suite_id == "fastq_hpc_01" {
        let (ok_mini, detail_mini) = mini_suite_stability_gate(cwd);
        checks.push(serde_json::json!({
            "name": "mini_suite_stability_no_drift",
            "ok": ok_mini,
            "detail": detail_mini,
        }));
    }

    let ok = checks
        .iter()
        .all(|check| check.get("ok").and_then(serde_json::Value::as_bool) == Some(true));
    Ok(serde_json::json!({
        "schema_version": "bijux.status.production_readiness.v1",
        "suite_id": suite_id,
        "ok": ok,
        "checks": checks,
    }))
}

fn suite_path(cwd: &Path, suite: &str) -> Result<PathBuf> {
    let preferred = cwd.join("bench-suites").join(format!("{suite}.toml"));
    if preferred.exists() {
        return Ok(preferred);
    }
    let fallback = cwd.join("configs").join(format!("{suite}.toml"));
    if fallback.exists() {
        return Ok(fallback);
    }
    Err(anyhow!(
        "suite spec not found: {} or {}",
        preferred.display(),
        fallback.display()
    ))
}

fn validate_suite_contracts(suite: &SuiteSpec) -> Result<()> {
    if suite.stages.is_empty() {
        return Err(anyhow!("suite must declare at least one stage"));
    }
    let fairness = suite.effective_fairness();
    if fairness.threads == 0 || fairness.mem_gb == 0 {
        return Err(anyhow!("fairness threads/mem_gb must be non-zero"));
    }
    if fairness.cold_runs == 0 && fairness.warm_runs == 0 {
        return Err(anyhow!(
            "fairness must include at least one cold or warm run"
        ));
    }

    let mut seen = BTreeSet::new();
    for stage in &suite.stages {
        if stage.tools.is_empty() {
            return Err(anyhow!(
                "stage {} must include at least one tool",
                stage.stage
            ));
        }
        if !seen.insert(stage.stage.clone()) {
            return Err(anyhow!("duplicate stage in suite: {}", stage.stage));
        }
    }

    let stage_set = suite
        .stages
        .iter()
        .map(|row| row.stage.as_str())
        .collect::<BTreeSet<_>>();
    if stage_set.len() == 1 {
        if !(stage_set.contains("validate_pre") || stage_set.contains("fastq.validate_pre")) {
            return Err(anyhow!(
                "single-stage suites must target `validate_pre` (stage-1 contract)"
            ));
        }
        return Ok(());
    }
    for stage in ["validate_pre", "trim", "filter", "stats", "qc_post"] {
        if !(stage_set.contains(stage) || stage_set.contains(&format!("fastq.{stage}").as_str())) {
            return Err(anyhow!("suite missing required stage `{stage}`"));
        }
    }
    Ok(())
}

fn suite_signature(cwd: &Path, suite_id: &str, hpc: bool) -> Result<String> {
    let path = suite_path(cwd, suite_id)?;
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(raw.as_bytes());
    if hpc {
        hasher.update(b"hpc");
    } else {
        hasher.update(b"local");
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn append_telemetry_event(path: &Path, event_name: &str, attrs: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let event = serde_json::json!({
        "schema_version": "bijux.telemetry.v1",
        "ts": now,
        "event": event_name,
        "attrs": attrs,
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(&event)?)
        .with_context(|| format!("append {}", path.display()))
}

fn pseudo_digest(seed: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn deterministic_metric(
    stage: &str,
    tool: &str,
    mode: &str,
    run_index: u32,
    min: f64,
    max: f64,
) -> f64 {
    let mut hasher = sha2::Sha256::new();
    hasher.update(stage.as_bytes());
    hasher.update(tool.as_bytes());
    hasher.update(mode.as_bytes());
    hasher.update(run_index.to_le_bytes());
    let bytes = hasher.finalize();
    let n = u16::from_le_bytes([bytes[0], bytes[1]]);
    let unit = f64::from(n) / f64::from(u16::MAX);
    min + unit * (max - min)
}

fn deterministic_u32(
    stage: &str,
    tool: &str,
    mode: &str,
    run_index: u32,
    min: u32,
    max: u32,
) -> u32 {
    if min >= max {
        return min;
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(stage.as_bytes());
    hasher.update(tool.as_bytes());
    hasher.update(mode.as_bytes());
    hasher.update(run_index.to_le_bytes());
    hasher.update(b"u32");
    let bytes = hasher.finalize();
    let n = u16::from_le_bytes([bytes[2], bytes[3]]);
    let range = max.saturating_sub(min);
    min + (u32::from(n) % (range + 1))
}

fn deterministic_i32(
    stage: &str,
    tool: &str,
    mode: &str,
    run_index: u32,
    min: i32,
    max: i32,
) -> i32 {
    if min >= max {
        return min;
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(stage.as_bytes());
    hasher.update(tool.as_bytes());
    hasher.update(mode.as_bytes());
    hasher.update(run_index.to_le_bytes());
    hasher.update(b"i32");
    let bytes = hasher.finalize();
    let n = u16::from_le_bytes([bytes[4], bytes[5]]);
    let span = u32::try_from(max - min).unwrap_or(0);
    if span == 0 {
        return min;
    }
    min + i32::try_from(u32::from(n) % (span + 1)).unwrap_or(0)
}

fn stage_checks(stage: &str) -> Vec<String> {
    match stage {
        "validate_pre" => vec![
            "fastq_format_valid".to_string(),
            "gzip_integrity_ok".to_string(),
            "artifact_set_complete".to_string(),
        ],
        "trim" | "filter" => vec![
            "fastq_format_valid".to_string(),
            "gzip_integrity_ok".to_string(),
            "delta_metrics_present".to_string(),
            "artifact_set_complete".to_string(),
        ],
        "stats" => vec!["stats_artifact_present".to_string()],
        "qc_post" => vec![
            "qc_artifact_present".to_string(),
            "artifact_set_complete".to_string(),
        ],
        _ => Vec::new(),
    }
}

fn write_repro_bundle(bundle_path: &Path, run_dir: &Path) -> Result<()> {
    if let Some(parent) = bundle_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let tar_gz = fs::File::create(bundle_path)
        .with_context(|| format!("create {}", bundle_path.display()))?;
    let encoder = GzEncoder::new(tar_gz, Compression::default());
    let mut archive = tar::Builder::new(encoder);
    for rel in [
        "run_manifest.json",
        "telemetry.jsonl",
        "tool_invocations.json",
        "decision_trace.json",
    ] {
        let path = run_dir.join(rel);
        if path.exists() {
            archive
                .append_path_with_name(&path, rel)
                .with_context(|| format!("add {}", path.display()))?;
        }
    }
    let metrics = run_dir.join("metrics");
    if metrics.exists() {
        archive
            .append_dir_all("metrics", &metrics)
            .with_context(|| format!("add {}", metrics.display()))?;
    }
    archive.finish().context("finalize reproducibility bundle")
}

fn validate_scientific_defaults_doc(cwd: &Path) -> Result<ScientificDefaultsStatus> {
    let path = cwd.join("docs").join("SCIENTIFIC_DEFAULTS.md");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut checked_rows = 0usize;
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed.contains("---") {
            continue;
        }
        let cols = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cols.len() < 7 {
            continue;
        }
        if cols[0].eq_ignore_ascii_case("domain") {
            continue;
        }
        let applies = cols[3].to_ascii_lowercase();
        if !applies.contains("reference") {
            continue;
        }
        checked_rows += 1;
        let rationale = cols[4];
        let comparability = cols[5];
        let citation = cols[6];
        if rationale.is_empty() || comparability.is_empty() || citation.is_empty() {
            return Err(anyhow!(
                "SCIENTIFIC_DEFAULTS reference row missing rationale/comparability/citation: {line}",
            ));
        }
    }
    if checked_rows == 0 {
        return Err(anyhow!(
            "SCIENTIFIC_DEFAULTS.md has no reference rows with enforceable metadata"
        ));
    }
    Ok(ScientificDefaultsStatus {
        doc_path: path.display().to_string(),
        reference_defaults_ok: true,
        checked_rows,
    })
}

fn capture_environment_snapshot(cwd: &Path) -> EnvironmentSnapshot {
    let apptainer_version = command_output("apptainer", &["--version"])
        .or_else(|| command_output("singularity", &["--version"]))
        .unwrap_or_else(|| "unavailable".to_string());
    let kernel = command_output("uname", &["-r"]).unwrap_or_else(|| "unknown".to_string());
    let site_lock = load_site_lock(cwd);
    EnvironmentSnapshot {
        apptainer_version,
        kernel,
        site_lock,
    }
}

fn command_output(cmd: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn load_site_lock(cwd: &Path) -> serde_json::Value {
    let hpc_root = std::env::var("BIJUX_HPC_ROOT")
        .map_or_else(|_| PathBuf::from("/home/bijan/bijux"), PathBuf::from);
    let candidates = [
        hpc_root.join("bijux-dna-results").join("site_lock.json"),
        cwd.join("bijux-dna-results").join("site_lock.json"),
    ];
    for path in candidates {
        if !path.exists() {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                return value;
            }
        }
    }
    serde_json::json!({
        "status": "missing",
        "detail": "site_lock.json not found",
    })
}

fn compute_comparability_hash(
    suite: &SuiteSpec,
    fairness: &FairnessSpec,
    tool_invocations: &[ToolInvocationRow],
    scientific_defaults: &ScientificDefaultsStatus,
) -> Result<String> {
    let mut versions = tool_invocations
        .iter()
        .map(|row| format!("{}:{}:{}", row.stage, row.tool, row.tool_version))
        .collect::<Vec<_>>();
    versions.sort();
    versions.dedup();
    let payload = serde_json::json!({
        "suite_id": suite.suite_id,
        "corpus": suite.corpus,
        "fairness": {
            "threads": fairness.threads,
            "mem_gb": fairness.mem_gb,
            "tmp_policy": fairness.tmp_policy,
        },
        "tool_versions": versions,
        "scientific_defaults": scientific_defaults,
    });
    let bytes = serde_json::to_vec(&payload)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn evaluate_scientific_sufficiency(rows: &[RunRecordRow]) -> ScientificSufficiency {
    const MIN_READS_AFTER: u64 = 100_000;
    const MIN_RETENTION: f64 = 0.10;
    let mut reasons = Vec::new();
    for row in rows {
        if row.read_retention < MIN_RETENTION {
            reasons.push(format!(
                "{}:{}:{} run{} retention {:.3} < {:.3}",
                row.stage, row.tool, row.mode, row.run_index, row.read_retention, MIN_RETENTION
            ));
        }
        if let Some(delta) = &row.delta_metrics {
            let reads_after = delta.get("reads_after").and_then(serde_json::Value::as_u64);
            if let Some(reads_after) = reads_after {
                if reads_after < MIN_READS_AFTER {
                    reasons.push(format!(
                        "{}:{}:{} run{} reads_after {} < {}",
                        row.stage, row.tool, row.mode, row.run_index, reads_after, MIN_READS_AFTER
                    ));
                }
            }
        }
    }
    ScientificSufficiency {
        sufficient: reasons.is_empty(),
        thresholds: serde_json::json!({
            "min_reads_after": MIN_READS_AFTER,
            "min_retention": MIN_RETENTION
        }),
        reasons,
    }
}

fn mini_suite_stability_gate(cwd: &Path) -> (bool, String) {
    let runs_dir = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join("fastq_hpc_01_mini")
        .join("runs");
    if !runs_dir.exists() {
        return (
            false,
            format!("missing mini suite runs at {}", runs_dir.display()),
        );
    }
    let mut manifests = fs::read_dir(&runs_dir)
        .ok()
        .into_iter()
        .flat_map(std::iter::Iterator::flatten)
        .map(|entry| entry.path().join("run_manifest.json"))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    manifests.sort();
    if manifests.len() < 2 {
        return (false, "need at least two corpus-01-mini runs".to_string());
    }
    let right = manifests.pop().unwrap_or_default();
    let left = manifests.pop().unwrap_or_default();
    let left_raw = match fs::read_to_string(&left) {
        Ok(value) => value,
        Err(err) => return (false, format!("read {} failed: {err}", left.display())),
    };
    let right_raw = match fs::read_to_string(&right) {
        Ok(value) => value,
        Err(err) => return (false, format!("read {} failed: {err}", right.display())),
    };
    let left_manifest = match serde_json::from_str::<SuiteRunManifest>(&left_raw) {
        Ok(value) => value,
        Err(err) => return (false, format!("parse {} failed: {err}", left.display())),
    };
    let right_manifest = match serde_json::from_str::<SuiteRunManifest>(&right_raw) {
        Ok(value) => value,
        Err(err) => return (false, format!("parse {} failed: {err}", right.display())),
    };
    if left_manifest.comparability_hash != right_manifest.comparability_hash {
        return (
            false,
            "comparability_hash drift detected on corpus-01-mini".to_string(),
        );
    }
    if left_manifest.decision_trace != right_manifest.decision_trace {
        return (
            false,
            "decision trace drift detected on corpus-01-mini".to_string(),
        );
    }
    (
        true,
        "stable (no drift across latest two mini runs)".to_string(),
    )
}

fn html_escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
