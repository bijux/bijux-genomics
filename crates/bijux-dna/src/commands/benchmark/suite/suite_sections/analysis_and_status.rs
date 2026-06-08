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
        bijux_dna_infra::write_string(&html_path, &html)
            .with_context(|| format!("write {}", html_path.display()))?;
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
        "ok": cwd.join("examples").join("bijux-dna-data").join(&suite.corpus).exists(),
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
            });
        let missing = seen
            .as_ref()
            .map(|seen| required_stages.difference(seen).cloned().collect::<Vec<_>>());
        checks.push(serde_json::json!({
            "name": "all_required_stages_ranked",
            "ok": missing.as_ref().is_some_and(Vec::is_empty),
            "detail": match missing {
                Some(missing) if missing.is_empty() => "ok".to_string(),
                Some(missing) => format!("missing: {}", missing.join(",")),
                None => "performance_ranking missing or not an array".to_string(),
            },
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
    let root = workspace_root();
    let preferred = bijux_dna_infra::bench_suites_dir(&root).join(format!("{suite}.toml"));
    if preferred.exists() {
        return Ok(preferred);
    }
    let fallback = bijux_dna_infra::configs_file(cwd, &format!("bench/{suite}.toml"));
    if fallback.exists() {
        return Ok(fallback);
    }
    Err(anyhow!(
        "suite spec not found: {} or {}",
        preferred.display(),
        fallback.display()
    ))
}

#[must_use]
pub fn bench_status(cwd: &Path) -> serde_json::Value {
    let root = workspace_root();
    let suite_dir = bijux_dna_infra::bench_suites_dir(&root);
    let config_dir = bijux_dna_infra::configs_dir(&root).join("bench");
    let mut suites = Vec::new();
    if suite_dir.exists() {
        if let Ok(entries) = fs::read_dir(&suite_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|v| v.to_str()) != Some("toml") {
                    continue;
                }
                if let Some(stem) = path.file_stem().and_then(|v| v.to_str()) {
                    suites.push(stem.to_string());
                }
            }
        }
    }
    suites.sort();
    suites.dedup();
    serde_json::json!({
        "schema_version": "bijux.bench.status.v1",
        "bench_suite_dir": suite_dir.display().to_string(),
        "bench_config_dir": config_dir.display().to_string(),
        "detected_suites": suites,
        "cwd": cwd.display().to_string(),
    })
}

fn workspace_root() -> PathBuf {
    crate::commands::support::workspace_root::resolve_repo_root()
        .unwrap_or_else(|err| panic!("{err}"))
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
        if !stage.stage.contains('.') {
            return Err(anyhow!(
                "stage {} must use canonical stage ids like fastq.trim_reads",
                stage.stage
            ));
        }
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

    validate_governed_stage_tool_bindings(suite)?;

    let stage_set = suite
        .stages
        .iter()
        .map(|row| row.stage.as_str())
        .collect::<BTreeSet<_>>();
    if stage_set.len() == 1 {
        return Ok(());
    }
    for stage_id in crate::commands::benchmark::alias_inventory::required_migration_stage_ids() {
        if !crate::commands::benchmark::alias_inventory::stage_set_contains_canonical_or_migration_alias(
            &stage_set,
            stage_id,
        ) {
            return Err(anyhow!("suite missing required stage `{stage_id}`"));
        }
    }
    Ok(())
}

fn validate_governed_stage_tool_bindings(suite: &SuiteSpec) -> Result<()> {
    let registry_path =
        bijux_dna_infra::configs_file(&workspace_root(), "ci/registry/tool_registry.toml");
    let registry = bijux_dna_api::v1::api::run::load_manifests(&registry_path)
        .with_context(|| format!("load {}", registry_path.display()))?;
    for stage in &suite.stages {
        let stage_id = bijux_dna_api::v1::api::run::StageId::try_from(stage.stage.as_str())
            .map_err(|err| anyhow!("invalid suite stage {}: {err}", stage.stage))?;
        let governed_tools = registry
            .tools_for_stage(&stage_id)
            .iter()
            .map(|tool| tool.tool_id.as_str())
            .collect::<BTreeSet<_>>();
        if governed_tools.is_empty() {
            return Err(anyhow!(
                "stage {} is not admitted in governed registry {}",
                stage.stage,
                registry_path.display()
            ));
        }
        for tool in &stage.tools {
            let tool_id = bijux_dna_api::v1::api::run::ToolId::try_from(tool.as_str())
                .map_err(|err| anyhow!("invalid tool id {} for stage {}: {err}", tool, stage.stage))?;
            if !governed_tools.contains(tool_id.as_str()) {
                return Err(anyhow!(
                    "stage {} tool {} is not admitted in governed registry",
                    stage.stage, tool
                ));
            }
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
    Ok(sha256_hex(&hasher.finalize()))
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
    format!("sha256:{}", sha256_hex(&hasher.finalize()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod suite_contract_tests {
    use super::{validate_suite_contracts, FairnessSpec, SuiteSpec, SuiteStage};

    fn governed_suite(stage: &str, tools: &[&str]) -> SuiteSpec {
        SuiteSpec {
            schema_version: "bijux.bench.suite.v1".to_string(),
            suite_id: "fastq_stage02_trim".to_string(),
            corpus: "example-102".to_string(),
            stages: vec![SuiteStage {
                stage: stage.to_string(),
                tools: tools.iter().map(|tool| (*tool).to_string()).collect(),
            }],
            repetitions: 2,
            resource_hints: None,
            fairness: Some(FairnessSpec {
                threads: 16,
                mem_gb: 64,
                tmp_policy: "unique-per-run-id".to_string(),
                cold_runs: 1,
                warm_runs: 1,
            }),
        }
    }

    #[test]
    fn suite_validation_rejects_noncanonical_stage_ids() {
        let suite = governed_suite("trim", &["fastp"]);
        let error = validate_suite_contracts(&suite).expect_err("legacy stage ids must fail");
        assert!(error.to_string().contains("canonical stage ids"));
    }

    #[test]
    fn suite_validation_rejects_unguarded_stage_tools() {
        let suite = governed_suite("fastq.trim_reads", &["imaginarytool"]);
        let error = validate_suite_contracts(&suite)
            .expect_err("planned or missing registry tool must fail");
        assert!(error.to_string().contains("not admitted in governed registry"));
    }

    #[test]
    fn suite_validation_accepts_governed_fastq_bindings() {
        let suite = governed_suite("fastq.trim_reads", &["fastp", "bbduk"]);
        validate_suite_contracts(&suite).expect("governed bindings must validate");
    }
}
