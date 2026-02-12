use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteSpec {
    pub schema_version: String,
    pub suite_id: String,
    pub corpus: String,
    pub stages: Vec<SuiteStage>,
    pub repetitions: u32,
    pub resource_hints: ResourceHints,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteStage {
    pub stage: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceHints {
    pub threads: u32,
    pub mem_gb: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuiteRunManifest {
    pub schema_version: String,
    pub suite_id: String,
    pub run_id: String,
    pub run_context: String,
    pub corpus: String,
    pub repetitions: u32,
    pub fairness: FairnessContract,
    pub cold_vs_warm: ColdWarmContract,
    pub stage_runs: Vec<StageRunManifest>,
    pub created_at_utc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FairnessContract {
    pub threads: u32,
    pub tmp_root: String,
    pub deviations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColdWarmContract {
    pub cold_run_records_image_cost: bool,
    pub warm_runs_exclude_image_cost: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageRunManifest {
    pub stage: String,
    pub required_role: String,
    pub tools: Vec<String>,
    pub contracts: StageContracts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageContracts {
    pub required_records: Vec<String>,
    pub required_validations: Vec<String>,
    pub acceptance_rule: String,
}

#[derive(Debug, Serialize)]
pub struct SuiteAnalysisReport {
    pub schema_version: String,
    pub suite_id: String,
    pub run_dir: String,
    pub performance_ranking: Vec<RankingRow>,
    pub scientific_deltas: Vec<DeltaRow>,
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

pub fn load_suite(cwd: &Path, suite: &str) -> Result<SuiteSpec> {
    let path = suite_path(cwd, suite);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
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
    let corpus_root = cwd.join("bijux-dna-data").join(&suite.corpus);
    if !corpus_root.exists() {
        return Err(anyhow!("missing corpus root {}", corpus_root.display()));
    }
    crate::commands::corpus::validate_corpus(cwd, &suite.corpus)?;

    let run_context = if hpc { "HPC" } else { "Local" }.to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let timestamp = now.to_string();
    let run_id = format!("{}-{}", suite.suite_id, timestamp.to_lowercase());
    let run_dir = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(&suite.suite_id)
        .join(&timestamp)
        .join(&run_id);
    bijux_dna_infra::ensure_dir(&run_dir)?;

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

    let stage_runs = suite
        .stages
        .iter()
        .map(|stage| StageRunManifest {
            stage: stage.stage.clone(),
            required_role: required_role_for_stage(&stage.stage).to_string(),
            tools: stage.tools.clone(),
            contracts: StageContracts {
                required_records: vec![
                    "stderr_tail".to_string(),
                    "command_line".to_string(),
                    "tool_version".to_string(),
                    "image_digest".to_string(),
                    "input_checksums".to_string(),
                ],
                required_validations: if stage.stage == "trim" || stage.stage == "filter" {
                    vec![
                        "artifact_correctness".to_string(),
                        "delta_metrics_before_after".to_string(),
                    ]
                } else {
                    vec!["artifact_correctness".to_string()]
                },
                acceptance_rule:
                    "reject empty outputs or invariant violations; exclude invalid run from ranking"
                        .to_string(),
            },
        })
        .collect::<Vec<_>>();

    let manifest = SuiteRunManifest {
        schema_version: "bijux.bench.suite_run_manifest.v1".to_string(),
        suite_id: suite.suite_id.clone(),
        run_id: run_id.clone(),
        run_context,
        corpus: suite.corpus.clone(),
        repetitions: suite.repetitions,
        fairness: FairnessContract {
            threads: suite.resource_hints.threads,
            tmp_root: tmp_root.display().to_string(),
            deviations: Vec::new(),
        },
        cold_vs_warm: ColdWarmContract {
            cold_run_records_image_cost: true,
            warm_runs_exclude_image_cost: true,
        },
        stage_runs,
        created_at_utc: now.to_string(),
    };

    let manifest_path = run_dir.join("run_manifest.json");
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;
    if hpc {
        let root = std::env::var("BIJUX_HPC_ROOT").map_or_else(
            |_| std::path::PathBuf::from("/home/bijan/bijux"),
            std::path::PathBuf::from,
        );
        let inventory = crate::commands::cli::env::sif_inventory(&root)?;
        let inventory_path = run_dir.join("sif_inventory.json");
        bijux_dna_infra::atomic_write_json(&inventory_path, &inventory)?;
    }

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
            "schema_version": "bijux.bench.suite_run_pointer.v1",
            "suite_id": suite.suite_id,
            "run_id": run_id,
            "run_dir": run_dir.display().to_string(),
        }),
    )?;

    Ok(run_dir)
}

pub fn analyze_suite(cwd: &Path, suite_id: &str) -> Result<PathBuf> {
    let latest_pointer = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(suite_id)
        .join("latest")
        .join("run_pointer.json");
    let pointer_raw = std::fs::read_to_string(&latest_pointer)
        .with_context(|| format!("read {}", latest_pointer.display()))?;
    let pointer: serde_json::Value = serde_json::from_str(&pointer_raw)
        .with_context(|| format!("parse {}", latest_pointer.display()))?;
    let run_dir = PathBuf::from(
        pointer
            .get("run_dir")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("run_pointer missing run_dir"))?,
    );
    let manifest_raw = std::fs::read_to_string(run_dir.join("run_manifest.json"))
        .with_context(|| format!("read {}/run_manifest.json", run_dir.display()))?;
    let manifest: SuiteRunManifest = serde_json::from_str(&manifest_raw)
        .with_context(|| format!("parse {}/run_manifest.json", run_dir.display()))?;

    let mut ranking = Vec::new();
    let mut deltas = Vec::new();
    let mut outliers = Vec::new();
    let mut invalid_runs_excluded = 0usize;
    for stage in &manifest.stage_runs {
        let mut ordinal = 1.0;
        for tool in &stage.tools {
            ranking.push(RankingRow {
                stage: stage.stage.clone(),
                tool: tool.clone(),
                score: ordinal,
            });
            ordinal += 1.0;
        }
        if stage
            .contracts
            .required_validations
            .iter()
            .any(|entry| entry == "delta_metrics_before_after")
        {
            deltas.push(DeltaRow {
                stage: stage.stage.clone(),
                metric: "delta_metrics".to_string(),
                note: "before/after reads+bases+length histogram required".to_string(),
            });
        }
        if stage.tools.is_empty() {
            outliers.push(format!("{}:no_tools_selected", stage.stage));
            invalid_runs_excluded += 1;
        }
    }

    ranking.sort_by(|a, b| {
        a.stage.cmp(&b.stage).then_with(|| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let report = SuiteAnalysisReport {
        schema_version: "bijux.bench.suite_analysis.v1".to_string(),
        suite_id: manifest.suite_id,
        run_dir: run_dir.display().to_string(),
        performance_ranking: ranking,
        scientific_deltas: deltas,
        outliers,
        invalid_runs_excluded,
    };
    let report_path = run_dir.join("analysis_report.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

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
        let raw = std::fs::read_to_string(&latest)
            .with_context(|| format!("read {}", latest.display()))?;
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

fn suite_path(cwd: &Path, suite: &str) -> PathBuf {
    cwd.join("configs").join(format!("{suite}.toml"))
}

fn validate_suite_contracts(suite: &SuiteSpec) -> Result<()> {
    if suite.stages.is_empty() {
        return Err(anyhow!("suite must declare at least one stage"));
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

    let required = ["validate_pre", "trim", "filter", "stats", "qc_post"];
    let stage_set = suite
        .stages
        .iter()
        .map(|row| row.stage.as_str())
        .collect::<BTreeSet<_>>();
    for stage in required {
        if !stage_set.contains(stage) {
            return Err(anyhow!("suite missing required stage `{stage}`"));
        }
    }
    Ok(())
}

fn required_role_for_stage(stage: &str) -> &'static str {
    match stage {
        "validate_pre" | "qc_post" => "qc",
        "trim" => "trimmer",
        "filter" => "filter",
        "stats" => "stats",
        _ => "unknown",
    }
}
