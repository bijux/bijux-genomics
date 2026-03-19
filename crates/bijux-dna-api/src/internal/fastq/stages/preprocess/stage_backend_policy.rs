fn canonical_sample_identity(sample_id: &str) -> String {
    let mut out = String::with_capacity(sample_id.len());
    for ch in sample_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}

fn parse_low_complexity_filtered_count(stdout: &str, stderr: &str) -> Option<u64> {
    let haystack = format!("{stdout}\n{stderr}");
    for line in haystack.lines() {
        if line.to_ascii_lowercase().contains("filtered") {
            let digits: String = line.chars().filter(char::is_ascii_digit).collect();
            if let Ok(parsed) = digits.parse::<u64>() {
                return Some(parsed);
            }
        }
    }
    None
}

fn parse_first_u64_after_key(text: &str, key: &str) -> Option<u64> {
    for line in text.lines() {
        if !line.to_ascii_lowercase().contains(&key.to_ascii_lowercase()) {
            continue;
        }
        let digits: String = line.chars().filter(char::is_ascii_digit).collect();
        if let Ok(parsed) = digits.parse::<u64>() {
            return Some(parsed);
        }
    }
    None
}

fn parse_validate_reads_metrics(execution: &StageResultV1) -> serde_json::Value {
    let merged = format!("{}\n{}", execution.stdout, execution.stderr);
    let read_count = parse_first_u64_after_key(&merged, "read")
        .or_else(|| parse_first_u64_after_key(&merged, "sequences"));
    let base_count =
        parse_first_u64_after_key(&merged, "base").or_else(|| parse_first_u64_after_key(&merged, "bp"));
    let errors = parse_first_u64_after_key(&merged, "error");
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.validate_reads",
        "validator": "tool_stdout_stderr_parser",
        "read_count": read_count,
        "base_count": base_count,
        "error_count": errors,
        "strict_pass": execution.exit_code == 0,
    })
}

fn parse_detect_adapters_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let fastp_json = out_dir.join("fastp.json");
    if let Ok(raw) = std::fs::read_to_string(&fastp_json) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
            let adapter_cut = parsed
                .pointer("/adapter_cutting/adapter_trimmed_reads")
                .and_then(serde_json::Value::as_u64);
            let total = parsed
                .pointer("/summary/before_filtering/total_reads")
                .and_then(serde_json::Value::as_u64);
            let fraction = match (adapter_cut, total) {
                (Some(cut), Some(t)) if t > 0 => {
                    let cut_f = cut.to_string().parse::<f64>().ok();
                    let total_f = t.to_string().parse::<f64>().ok();
                    match (cut_f, total_f) {
                        (Some(c), Some(total_reads)) => Some(c / total_reads),
                        _ => None,
                    }
                }
                _ => None,
            };
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.detect_adapters",
                "adapter_inference": {
                    "source": "fastp",
                    "adapter_trimmed_reads": adapter_cut,
                    "reads_total": total,
                    "adapter_trimmed_fraction": fraction,
                }
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.detect_adapters",
        "adapter_inference": {
            "detected": out_dir.join("fastqc").exists(),
            "source": "stage_outputs",
            "output_dir": out_dir.join("fastqc"),
        },
    })
}

fn stage_network_policy(stage_id: &str) -> NetworkPolicy {
    match stage_id {
        "fastq.validate_reads"
        | "fastq.detect_adapters"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_reads"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.correct_errors"
        | "fastq.filter_reads"
        | "fastq.filter_low_complexity"
        | "fastq.trim_polyg_tails"
        | "fastq.screen_taxonomy" => NetworkPolicy::Forbid,
        _ => NetworkPolicy::Allow,
    }
}

fn enforce_fastq_backend_allowlist(stage_id: &str, tool_id: &str) -> Result<()> {
    let allowed: &[&str] = match stage_id {
        "fastq.validate_reads" => &["fastqvalidator", "fqtools", "seqtk"],
        "fastq.detect_adapters" => &["fastp", "fastqc"],
        "fastq.trim_reads" => &[
            "alientrimmer",
            "bbduk",
            "cutadapt",
            "fastp",
            "fastx_clipper",
            "seqkit",
        ],
        "fastq.trim_terminal_damage" => &["cutadapt", "seqkit"],
        "fastq.merge_pairs" => &["bbmerge", "flash2", "leehom", "pear"],
        "fastq.remove_duplicates" => &["clumpify", "fastuniq"],
        "fastq.correct_errors" => &["lighter", "rcorrector", "musket", "spades", "bayeshammer"],
        "fastq.filter_reads" => &["bbduk", "fastp", "prinseq", "seqkit"],
        "fastq.filter_low_complexity" => &["bbduk", "prinseq", "dustmasker"],
        "fastq.trim_polyg_tails" => &["fastp", "bbduk"],
        "fastq.screen_taxonomy" => &["kraken2", "bracken", "centrifuge", "kaiju", "metaphlan", "krakenuniq", "fastq_screen"],
        "fastq.deplete_reference_contaminants" => &["bbduk", "bowtie2"],
        "fastq.deplete_rrna" => &["sortmerna"],
        "fastq.deplete_host" => &["bowtie2"],
        "fastq.normalize_primers" => &["cutadapt"],
        "fastq.remove_chimeras" | "fastq.cluster_otus" => &["vsearch"],
        "fastq.infer_asvs" => &["dada2"],
        "fastq.normalize_abundance" => &["seqfu", "seqkit"],
        _ => return Ok(()),
    };
    if allowed.contains(&tool_id) {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported backend for {stage_id}: `{tool_id}` not in allowlist {}",
        allowed.join(",")
    ))
}

fn workspace_root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
}

fn required_fastq_tools() -> Result<std::collections::BTreeSet<String>> {
    let raw = std::fs::read_to_string(
        workspace_root_path().join("configs/ci/tools/required_tools.toml"),
    )?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let mut set = std::collections::BTreeSet::new();
    let items = parsed
        .get("required_tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing required_tools in required_tools.toml"))?;
    for item in items {
        if let Some(id) = item.as_str() {
            set.insert(id.to_string());
        }
    }
    Ok(set)
}

fn enforce_screen_db_governance(planned: &ExecutionStep) -> Result<()> {
    let stage = planned.step_id.as_str();
    if !matches!(
        stage,
        "fastq.screen_taxonomy" | "fastq.deplete_rrna" | "fastq.deplete_host" | "fastq.deplete_reference_contaminants"
    ) {
        return Ok(());
    }
    let template = planned.command.template.join(" ");
    if template.contains("http://") || template.contains("https://") {
        return Err(anyhow!(
            "{stage} may not fetch databases over network at runtime; use pre-mounted references"
        ));
    }
    if template.contains("download") || template.contains("pull") {
        return Err(anyhow!(
            "{stage} command contains database fetch verbs; require immutable pre-resolved DB paths"
        ));
    }
    Ok(())
}

fn required_metrics_keys(stage_id: &str) -> &'static [&'static str] {
    match stage_id {
        "fastq.validate_reads" => &["schema_version", "stage", "strict_pass"],
        "fastq.detect_adapters" => &["schema_version", "stage", "adapter_inference"],
        "fastq.trim_reads" => &["schema_version", "stage", "tool", "input_reads", "output_reads"],
        "fastq.trim_terminal_damage" => &[
            "schema_version",
            "stage",
            "udg_classification",
            "ct_ga_asymmetry_pre",
            "ct_ga_asymmetry_post",
        ],
        "fastq.merge_pairs" => &["schema_version", "stage", "tool", "paired_input", "merged_output"],
        "fastq.remove_duplicates" => &["schema_version", "stage", "tool", "duplicates_removed"],
        "fastq.correct_errors" => &["schema_version", "stage", "tool", "corrected_reads"],
        "fastq.filter_reads" => &["schema_version", "stage", "tool", "filtered_reads"],
        "fastq.filter_low_complexity" => &["schema_version", "stage", "tool", "low_complexity_removed"],
        "fastq.trim_polyg_tails" => &["schema_version", "stage", "tool", "trimmed_reads"],
        "fastq.screen_taxonomy" => &["schema_version", "stage", "tool", "taxonomy_profile"],
        "fastq.deplete_reference_contaminants" => &["schema_version", "stage", "tool", "screening_results"],
        "fastq.deplete_host" => &["schema_version", "stage", "tool", "host_removed_fraction"],
        _ => &["schema_version", "stage"],
    }
}

fn enforce_metrics_schema(stage_root: &std::path::Path, stage_id: &str) -> Result<()> {
    let metrics_path = stage_root.join("metrics.json");
    let raw = std::fs::read_to_string(&metrics_path)
        .with_context(|| format!("reading metrics {}", metrics_path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing metrics {}", metrics_path.display()))?;
    let required = required_metrics_keys(stage_id);
    for key in required {
        if parsed.get(*key).is_none() {
            return Err(anyhow!(
                "metrics schema violation for {stage_id}: missing key `{key}` in {}",
                metrics_path.display()
            ));
        }
    }
    Ok(())
}

fn count_fastq_reads_if_plain(path: &std::path::Path) -> Option<u64> {
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or_default();
    if ext == "gz" {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let lines = std::io::BufReader::new(file).lines().count() as u64;
    Some(lines / 4)
}

fn write_retention_report(stage_root: &std::path::Path, planned: &ExecutionStep) -> Result<()> {
    let out_dir = stage_root.join("out");
    let mut rows = vec!["artifact\treads_estimate".to_string()];
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let reads = count_fastq_reads_if_plain(&path)
                .map_or_else(|| "na".to_string(), |x| x.to_string());
            rows.push(format!("{name}\t{reads}"));
        }
    }
    let payload = rows.join("\n") + "\n";
    std::fs::write(stage_root.join("retention_report.tsv"), payload)?;
    std::fs::write(
        stage_root.join("retention_report.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.fastq.retention_report.v1",
            "stage_id": planned.step_id.0,
            "out_dir": out_dir,
            "artifacts": rows.len().saturating_sub(1),
        }))?,
    )?;
    Ok(())
}

fn classify_failure_hint(stage_id: &str, stdout: &str, stderr: &str) -> String {
    let merged = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    if merged.contains("out of memory") || merged.contains("killed") {
        return "resource_exhausted_memory".to_string();
    }
    if merged.contains("no space left") {
        return "resource_exhausted_disk".to_string();
    }
    if merged.contains("permission denied") {
        return "filesystem_permissions".to_string();
    }
    if merged.contains("not found") || merged.contains("no such file") {
        return "missing_input_or_tool".to_string();
    }
    format!("{stage_id}_execution_failure")
}

fn write_retry_policy(root: &std::path::Path) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": "bijux.retry_policy.v1",
        "max_retries": 0,
        "note": "fastq preprocessing stages are deterministic and should not auto-retry by default"
    });
    std::fs::write(root.join("retry_policy.json"), serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}
