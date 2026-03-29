fn write_stage_resume_contract(
    stage_root: &std::path::Path,
    stage_id: &str,
    execution: &StageResultV1,
    resumed: bool,
) -> Result<()> {
    let mut checksums = serde_json::Map::new();
    for path in &execution.outputs {
        let key = path
            .file_name()
            .and_then(|x| x.to_str())
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| path.display().to_string());
        let value = if path.exists() {
            bijux_dna_infra::hash_file_sha256(path).ok().map_or(
                serde_json::Value::Null,
                serde_json::Value::String,
            )
        } else {
            serde_json::Value::Null
        };
        checksums.insert(key, value);
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.stage_resume_contract.v1",
        "stage_id": stage_id,
        "resumed": resumed,
        "exit_code": execution.exit_code,
        "output_count": execution.outputs.len(),
        "outputs_sha256": checksums
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.resume_contract.json"), &payload)
        .context("write stage.resume_contract.json")
}

fn write_merge_join_contract(
    stage_root: &std::path::Path,
    execution: &StageResultV1,
    paired_consistent: bool,
) -> Result<()> {
    let expected_files = [
        "merged.fastq.gz",
        "unmerged_R1.fastq.gz",
        "unmerged_R2.fastq.gz",
    ];
    let emitted_names = execution
        .outputs
        .iter()
        .filter_map(|x| {
            x.file_name()
                .and_then(|n| n.to_str())
                .map(ToString::to_string)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let required_artifacts_present = expected_files
        .iter()
        .all(|name| emitted_names.contains(*name));
    let success = execution.exit_code == 0 && paired_consistent && required_artifacts_present;
    let failure_reason = if success {
        None
    } else if execution.exit_code != 0 {
        Some("merge tool exited non-zero".to_string())
    } else if !paired_consistent {
        Some("paired-end input consistency check failed".to_string())
    } else {
        Some("required merge artifacts missing".to_string())
    };
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.merge_pairs_join_contract.v1",
        "stage_id": "fastq.merge_pairs",
        "success": success,
        "criteria": {
            "exit_code_zero": execution.exit_code == 0,
            "paired_input_consistent": paired_consistent,
            "outputs_emitted": !execution.outputs.is_empty(),
            "required_artifacts_present": required_artifacts_present,
        },
        "required_artifacts": expected_files,
        "failure_reason": failure_reason,
        "artifacts": execution.outputs,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("merge.join_contract.json"), &payload)
        .context("write merge.join_contract.json")
}

fn load_qc_thresholds_map() -> std::collections::BTreeMap<String, f64> {
    let Some(path) = std::env::var_os("BIJUX_QC_THRESHOLDS_PATH")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("BIJUX_REFERENCE_ROOT")
                .map(std::path::PathBuf::from)
                .map(|root| root.join("qc_thresholds.yaml"))
        })
    else {
        return std::collections::BTreeMap::new();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return std::collections::BTreeMap::new();
    };
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || !line.contains(':') {
                return None;
            }
            let (k, v) = line.split_once(':')?;
            let key = k.trim().to_string();
            let value = v.trim().parse::<f64>().ok()?;
            Some((key, value))
        })
        .collect()
}

fn copy_if_missing(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    if dst.exists() {
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    std::fs::copy(src, dst)
        .with_context(|| format!("copy {} -> {}", src.display(), dst.display()))?;
    Ok(())
}

fn command_exists(bin: &str) -> bool {
    let args = vec!["--version".to_string()];
    bijux_dna_runner::command_runner::run_command(bin, &args).is_ok()
}

fn run_stage_command(
    out_dir: &std::path::Path,
    command_label: &str,
    bin: &str,
    args: &[String],
) -> bool {
    let output = bijux_dna_runner::command_runner::run_command(bin, args);
    let (ok, stdout, stderr) = match output {
        Ok(out) => (
            out.exit_code == 0,
            out.stdout,
            out.stderr,
        ),
        Err(err) => (false, String::new(), format!("{err}")),
    };
    let payload = format!(
        "label={command_label}\ncmd={} {}\nok={ok}\n--- stdout ---\n{}\n--- stderr ---\n{}\n",
        bin,
        args.join(" "),
        stdout,
        stderr
    );
    let _ = bijux_dna_infra::atomic_write_bytes(
        &out_dir.join(format!("{command_label}.command.log")),
        payload.as_bytes(),
    );
    ok
}

fn write_fastq_to_fasta_if_missing(
    input_fastq: &std::path::Path,
    out_fasta: &std::path::Path,
) -> Result<()> {
    if out_fasta.exists() {
        return Ok(());
    }
    if command_exists("seqkit") {
        let ok = run_stage_command(
            out_fasta
                .parent()
                .unwrap_or_else(|| std::path::Path::new(".")),
            "seqkit_fq2fa",
            "seqkit",
            &[
                "fq2fa".to_string(),
                input_fastq.to_string_lossy().to_string(),
                "-o".to_string(),
                out_fasta.to_string_lossy().to_string(),
            ],
        );
        if ok && out_fasta.exists() {
            return Ok(());
        }
    }
    // Deterministic fallback converter for basic FASTQ input.
    let mut out = String::new();
    let mut it = open_fastq_lines(input_fastq)?;
    while let (Some(h), Some(seq), Some(_plus), Some(_qual)) =
        (it.next(), it.next(), it.next(), it.next())
    {
        let header = h.trim_start_matches('@');
        out.push('>');
        out.push_str(header);
        out.push('\n');
        out.push_str(seq.trim());
        out.push('\n');
    }
    bijux_dna_infra::atomic_write_bytes(out_fasta, out.as_bytes())?;
    Ok(())
}

fn infer_udg_classification(input: &std::path::Path) -> String {
    if let Ok(configured) = std::env::var("BIJUX_UDG_CLASSIFICATION") {
        let normalized = configured.trim().to_ascii_lowercase();
        if matches!(normalized.as_str(), "udg" | "partial" | "non_udg") {
            return normalized;
        }
    }
    let stem = input
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if stem.contains("partial_udg") || stem.contains("partial-udg") {
        "partial".to_string()
    } else if stem.contains("udg") {
        "udg".to_string()
    } else {
        "non_udg".to_string()
    }
}

fn terminal_damage_profile(path: &std::path::Path) -> Result<serde_json::Value> {
    let mut ct_events = 0_u64;
    let mut ga_events = 0_u64;
    let mut seen = 0_u64;
    let mut five_prime: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut three_prime: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut lines = open_fastq_lines(path)?;
    while let (Some(_h), Some(seq), Some(_plus), Some(_qual)) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        let seq = seq.trim().to_ascii_uppercase();
        if seq.len() < 2 {
            continue;
        }
        let first = seq.chars().next().unwrap_or('N');
        let last = seq.chars().next_back().unwrap_or('N');
        *five_prime.entry(first.to_string()).or_insert(0) += 1;
        *three_prime.entry(last.to_string()).or_insert(0) += 1;
        if seq.starts_with("CT") {
            ct_events += 1;
        }
        if seq.ends_with("GA") {
            ga_events += 1;
        }
        seen += 1;
        if seen >= 200_000 {
            break;
        }
    }
    let denom = (ct_events + ga_events)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);
    let asymmetry = if denom > 0.0 {
        (ct_events.to_string().parse::<f64>().unwrap_or(0.0)
            - ga_events.to_string().parse::<f64>().unwrap_or(0.0))
            / denom
    } else {
        0.0
    };
    Ok(serde_json::json!({
        "reads_profiled": seen,
        "terminal_base_composition_5p": five_prime,
        "terminal_base_composition_3p": three_prime,
        "ct_events": ct_events,
        "ga_events": ga_events,
        "ct_ga_asymmetry": asymmetry,
    }))
}

fn enforce_stage_applicability(
    planned: &ExecutionStep,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let stage = planned.step_id.as_str();
    if stage == "fastq.merge_pairs" && args.r2.is_none() {
        return Err(anyhow!(
            "stage fastq.merge_pairs requires paired-end input (missing R2)"
        ));
    }
    if matches!(
        stage,
        "fastq.normalize_primers"
            | "fastq.remove_chimeras"
            | "fastq.infer_asvs"
            | "fastq.cluster_otus"
            | "fastq.normalize_abundance"
    ) && !matches!(
        args.mode,
        bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
            | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
    ) {
        return Err(anyhow!(
            "stage {stage} is only applicable in eDNA/pollen amplicon modes"
        ));
    }
    if stage == "fastq.deplete_reference_contaminants" {
        let template = planned.command.template.join(" ");
        let contaminant_root = declared_contaminant_asset_root()?;
        if !template.contains(&contaminant_root.display().to_string()) {
            return Err(anyhow!(
                "fastq.deplete_reference_contaminants requires contaminant assets under {}",
                contaminant_root.display()
            ));
        }
        if contaminant_bank.is_none() {
            return Err(anyhow!(
                "fastq.deplete_reference_contaminants requires contaminant bank context"
            ));
        }
    }
    Ok(())
}

fn declared_contaminant_asset_root() -> Result<std::path::PathBuf> {
    if let Some(root) = std::env::var_os("BIJUX_CONTAMINANT_ROOT")
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
    {
        return Ok(root);
    }
    if let Some(root) = std::env::var_os("BIJUX_REFERENCE_ROOT")
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
    {
        return Ok(root.join("contaminants"));
    }
    Err(anyhow!(
        "BIJUX_CONTAMINANT_ROOT or BIJUX_REFERENCE_ROOT must be declared for contaminant governance"
    ))
}

fn write_stage_governance_artifacts(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let stage = planned.step_id.as_str();
    if !matches!(
        stage,
        "fastq.screen_taxonomy" | "fastq.deplete_rrna" | "fastq.deplete_host" | "fastq.deplete_reference_contaminants"
    ) {
        return Ok(());
    }
    let template = planned.command.template.join(" ");
    let lower = template.to_ascii_lowercase();
    let db_flags_present = [
        " --db ",
        "--database",
        "--index",
        "kraken_db",
        "db_path",
        "--ref",
        "--reference",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.governance.v1",
        "stage_id": stage,
        "db_flags_present": db_flags_present,
        "command_template": planned.command.template,
        "contaminant_bank": if stage == "fastq.deplete_reference_contaminants" { contaminant_bank.cloned() } else { None::<serde_json::Value> },
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.governance.json"), &payload)
        .context("write stage.governance.json")
}

fn write_fastq_output_contract(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
    execution: &StageResultV1,
) -> Result<()> {
    let declared_outputs = planned
        .io
        .outputs
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "name": artifact.name,
                "role": artifact.role.as_str(),
                "path": artifact.path,
            })
        })
        .collect::<Vec<_>>();
    let emitted_outputs = execution
        .outputs
        .iter()
        .map(|path| serde_json::json!({ "path": path }))
        .collect::<Vec<_>>();
    let expected_ecological_outputs = match planned.stage_id.as_str() {
        "fastq.trim_terminal_damage" => {
            vec!["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
        }
        "fastq.normalize_primers" => vec!["primer_orientation_report"],
        "fastq.remove_chimeras" => vec!["report_json", "chimera_metrics_json"],
        "fastq.infer_asvs" => vec!["asv_table_tsv", "asv_sequences_fasta", "report_json"],
        "fastq.cluster_otus" => vec!["otu_table", "otu_representatives"],
        "fastq.normalize_abundance" => vec!["normalized_abundance_tsv"],
        _ => Vec::new(),
    };
    let ecological_checksums = planned
        .io
        .outputs
        .iter()
        .filter(|artifact| {
            expected_ecological_outputs
                .iter()
                .any(|name| *name == artifact.name.as_str())
        })
        .map(|artifact| {
            let sha256 = if artifact.path.exists() {
                bijux_dna_infra::hash_file_sha256(&artifact.path).ok()
            } else {
                None
            };
            serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "sha256": sha256
            })
        })
        .collect::<Vec<_>>();
    let contract = serde_json::json!({
        "schema_version": "bijux.fastq.output_contract.v1",
        "stage_id": planned.stage_id,
        "step_id": planned.step_id,
        "declared_outputs": declared_outputs,
        "emitted_outputs": emitted_outputs,
        "expected_ecological_outputs": expected_ecological_outputs,
        "ecological_output_checksums": ecological_checksums,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.output.contract.json"), &contract)
        .context("write stage output contract")
}

fn write_taxonomy_db_drift_report(
    run_root: &std::path::Path,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let report_path = run_root.join("taxonomy_db_drift.json");
    let current = contaminant_bank
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let lock_path = run_root.join("taxonomy_db.lock.json");
    let previous = if lock_path.exists() {
        let raw = std::fs::read_to_string(&lock_path).unwrap_or_default();
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let current_hash = bijux_dna_core::prelude::params_hash(&current).unwrap_or_default();
    let previous_hash = previous
        .get("current_hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    let drift_detected = lock_path.exists() && current_hash != previous_hash;
    let report = serde_json::json!({
        "schema_version": "bijux.taxonomy_db_drift.v1",
        "drift_detected": drift_detected,
        "current_hash": current_hash,
        "previous_hash": previous_hash,
        "current": current,
    });
    bijux_dna_infra::atomic_write_json(&report_path, &report).context("write taxonomy_db_drift")?;
    bijux_dna_infra::atomic_write_json(&lock_path, &report).context("write taxonomy_db lock")?;
    Ok(())
}

include!("pipeline_run.rs");
