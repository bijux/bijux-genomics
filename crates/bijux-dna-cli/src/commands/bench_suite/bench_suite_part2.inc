
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
    let path = cwd
        .join("docs")
        .join("20-science")
        .join("SCIENTIFIC_DEFAULTS.md");
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
    let hpc_root = crate::commands::hpc::load_hpc_config()
        .map(|cfg| cfg.resolve_paths().root)
        .unwrap_or(PathBuf::from("bijux"));
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
