fn warnings_for_plan(plan: &StagePlanV1, params: &serde_json::Value) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(msg) = polyx_unsupported_warning(plan.tool_id.0.as_str(), params) {
        warnings.push(msg);
    }
    if plan.stage_id.0 == "fastq.filter" {
        let redundant_filters = params
            .get("redundant_filters")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<&str>>()
            })
            .unwrap_or_default();
        if !redundant_filters.is_empty() {
            warnings.push(format!(
                "warning: filter stage received redundant filters already handled in trim: {}",
                redundant_filters.join(", ")
            ));
        }
    }
    if params.get("kmer_ref").is_some() && !tool_supports_kmer_filter(plan.tool_id.0.as_str()) {
        warnings.push(format!(
            "warning: k-mer filter requested but tool '{}' does not advertise k-mer support",
            plan.tool_id.0
        ));
    }
    if let Some(redundant) = params
        .get("redundant_filters")
        .and_then(|value| value.as_array())
    {
        if !redundant.is_empty() {
            let rendered: Vec<String> = redundant
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect();
            if !rendered.is_empty() {
                warnings.push(format!(
                    "warning: filter may be redundant; already handled by trim: {}",
                    rendered.join(", ")
                ));
            }
        }
    }
    warnings
}

fn quality_gate_decision(
    stage_id: &str,
    metrics: &serde_json::Value,
    reads_in: Option<u64>,
    reads_out: Option<u64>,
) -> Option<serde_json::Value> {
    if !matches!(
        stage_id,
        "fastq.trim" | "fastq.filter" | "fastq.qc_post" | "fastq.validate_pre"
    ) {
        return None;
    }
    let mut status = "pass".to_string();
    let mut reasons = Vec::new();
    let read_retention = metrics
        .get("delta_metrics")
        .and_then(|value| value.get("read_retention"))
        .and_then(serde_json::Value::as_f64)
        .or_else(|| {
            if let (Some(r_in), Some(r_out)) = (reads_in, reads_out) {
                if r_in > 0 {
                    return Some(f64_from_u64(r_out) / f64_from_u64(r_in));
                }
            }
            None
        });
    if let Some(retention) = read_retention {
        if retention < 0.4 {
            status = "fail".to_string();
            reasons.push(format!("read_retention {retention:.2} < 0.4"));
        } else if retention < 0.7 {
            status = "warn".to_string();
            reasons.push(format!("read_retention {retention:.2} < 0.7"));
        }
    }
    let mean_q = metrics.get("mean_q").and_then(serde_json::Value::as_f64);
    if let Some(mean_q) = mean_q {
        if mean_q < 15.0 {
            status = "fail".to_string();
            reasons.push(format!("mean_q {mean_q:.1} < 15"));
        } else if mean_q < 20.0 {
            status = "warn".to_string();
            reasons.push(format!("mean_q {mean_q:.1} < 20"));
        }
    }
    let mean_q_delta = metrics
        .get("delta_metrics")
        .and_then(|value| value.get("mean_q_delta"))
        .and_then(serde_json::Value::as_f64);
    if let Some(delta) = mean_q_delta {
        if delta < -1.0 {
            status = "warn".to_string();
            reasons.push(format!("mean_q_delta {delta:.2} < -1"));
        }
    }
    Some(serde_json::json!({
        "schema_version": "bijux.quality_gate.v1",
        "stage_id": stage_id,
        "status": status,
        "reasons": reasons,
        "thresholds": {
            "read_retention_warn": 0.7,
            "read_retention_fail": 0.4,
            "mean_q_warn": 20.0,
            "mean_q_fail": 15.0,
            "mean_q_delta_warn": -1.0
        }
    }))
}

#[derive(Debug, Default, Clone)]
#[allow(clippy::struct_field_names)]
struct FilterRemovalCounts {
    by_n: u64,
    by_entropy: u64,
    by_low_complexity: u64,
    by_kmer: u64,
    by_contaminant_kmer: u64,
    by_length: u64,
}

fn filter_removals_from_fastp(path: &Path) -> Option<FilterRemovalCounts> {
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let filtering = parsed.get("filtering_result")?;
    let by_n = filtering
        .get("too_many_N_reads")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let by_entropy = filtering
        .get("low_complexity_reads")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let by_length = filtering
        .get("too_short_reads")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        + filtering
            .get("too_long_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
    Some(FilterRemovalCounts {
        by_n,
        by_entropy,
        by_low_complexity: by_entropy,
        by_kmer: 0,
        by_contaminant_kmer: 0,
        by_length,
    })
}

fn filter_removals_from_bbduk_stats(
    path: &Path,
    kmer_ref_used: bool,
) -> Option<FilterRemovalCounts> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut removed = None;
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("Reads Removed") || line.starts_with("Reads removed") {
            let digits: String = line.chars().filter(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                removed = digits.parse::<u64>().ok();
            }
        }
    }
    let removed = removed?;
    Some(FilterRemovalCounts {
        by_n: 0,
        by_entropy: 0,
        by_low_complexity: 0,
        by_kmer: if kmer_ref_used { removed } else { 0 },
        by_contaminant_kmer: if kmer_ref_used { removed } else { 0 },
        by_length: 0,
    })
}

fn filter_removals_for_plan(
    tool_id: &str,
    out_dir: &Path,
    params: &serde_json::Value,
) -> FilterRemovalCounts {
    match tool_id {
        "fastp" => filter_removals_from_fastp(&out_dir.join("fastp.json")).unwrap_or_default(),
        "bbduk" => {
            let kmer_ref_used = params.get("kmer_ref").is_some();
            filter_removals_from_bbduk_stats(&out_dir.join("bbduk.stats"), kmer_ref_used)
                .unwrap_or_default()
        }
        _ => FilterRemovalCounts::default(),
    }
}

type IoDeltas = (
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
);

fn extract_io_deltas(metrics: &serde_json::Value) -> IoDeltas {
    let reads_in = metrics.get("reads_in").and_then(serde_json::Value::as_u64);
    let reads_out = metrics.get("reads_out").and_then(serde_json::Value::as_u64);
    let bases_in = metrics.get("bases_in").and_then(serde_json::Value::as_u64);
    let bases_out = metrics.get("bases_out").and_then(serde_json::Value::as_u64);
    let pairs_in = metrics.get("pairs_in").and_then(serde_json::Value::as_u64);
    let pairs_out = metrics.get("pairs_out").and_then(serde_json::Value::as_u64);
    (
        reads_in, reads_out, bases_in, bases_out, pairs_in, pairs_out,
    )
}

fn write_effective_fasta(
    run_artifacts_dir: &Path,
    name: &str,
    entries: &[BankEntryRecord],
    extra_fasta: &[String],
) -> Result<Option<(PathBuf, String)>> {
    if entries.is_empty() && extra_fasta.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    bijux_infra::ensure_dir(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.fasta"));
    let mut payload = String::new();
    for entry in entries {
        payload.push('>');
        payload.push_str(&entry.id);
        payload.push('\n');
        payload.push_str(&entry.sequence);
        payload.push('\n');
    }
    for fasta in extra_fasta {
        payload.push_str(fasta);
        if !fasta.ends_with('\n') {
            payload.push('\n');
        }
    }
    bijux_infra::atomic_write_bytes(&path, payload.as_bytes())
        .context("write effective bank fasta")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn bank_asset_name(bank_name: &str) -> &str {
    match bank_name {
        "adapter" => "adapters",
        "contaminant" => "contaminants",
        other => other,
    }
}

fn write_effective_bank_yaml(
    run_artifacts_dir: &Path,
    name: &str,
    bank_value: &serde_json::Value,
    entries: &[BankEntryRecord],
    references: &[BankReferenceRecord],
) -> Result<Option<(PathBuf, String)>> {
    if entries.is_empty() && references.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    bijux_infra::ensure_dir(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.yaml"));
    let payload = serde_json::json!({
        "bank_id": bank_value.get("bank_id"),
        "bank_hash": bank_value.get("bank_hash"),
        "preset": bank_value.get("preset"),
        "preset_hash": bank_value.get("preset_hash"),
        "enabled_entries": entries.iter().map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "sequence": entry.sequence,
                "rationale": entry.rationale,
                "source": entry.source,
            })
        }).collect::<Vec<_>>(),
        "references": references.iter().map(|reference| {
            serde_json::json!({
                "id": reference.id,
                "file": reference.file,
                "sha256": reference.sha256,
                "rationale": reference.rationale,
                "source": reference.source,
            })
        }).collect::<Vec<_>>(),
    });
    let yaml = serde_yaml::to_string(&payload).context("serialize effective bank yaml")?;
    bijux_infra::atomic_write_bytes(&path, yaml.as_bytes()).context("write effective bank yaml")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn write_effective_fasta_list(
    run_artifacts_dir: &Path,
    name: &str,
    references: &[BankReferenceRecord],
) -> Result<Option<(PathBuf, String)>> {
    if references.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    bijux_infra::ensure_dir(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.fasta.list"));
    let payload = references
        .iter()
        .map(|reference| reference.file.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    bijux_infra::atomic_write_bytes(&path, payload.as_bytes())
        .context("write effective bank fasta list")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn materialize_bank_assets(
    run_artifacts_dir: &Path,
    banks_value: Option<&serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    let Some(banks_value) = banks_value.and_then(|value| value.as_object()) else {
        return Ok(None);
    };
    let mut assets = serde_json::Map::new();
    for (bank_name, bank_value) in banks_value {
        let asset_name = bank_asset_name(bank_name);
        let entries = bank_entries_from_value(bank_value);
        let references = bank_references_from_value(bank_value);
        let extra_fasta: Vec<String> = references
            .iter()
            .filter_map(|reference| reference.fasta.clone())
            .collect();
        let fasta = write_effective_fasta(run_artifacts_dir, asset_name, &entries, &extra_fasta)?;
        let yaml = write_effective_bank_yaml(
            run_artifacts_dir,
            asset_name,
            bank_value,
            &entries,
            &references,
        )?;
        let fasta_list = if bank_name.as_str() == "contaminant" {
            write_effective_fasta_list(run_artifacts_dir, asset_name, &references)?
        } else {
            None
        };
        let record = serde_json::json!({
            "yaml": yaml.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
            "fasta": fasta.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
            "fasta_list": fasta_list.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
        });
        assets.insert(bank_name.clone(), record);
    }
    Ok(Some(serde_json::Value::Object(assets)))
}

fn fastq_stats(path: &Path) -> Result<bijux_core::measure::SeqkitMetrics> {
    let file = std::fs::File::open(path).context("open fastq")?;
    let reader: Box<dyn std::io::Read> = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let mut reads: u64 = 0;
    let mut bases: u64 = 0;
    let mut gc: u64 = 0;
    let mut q_sum: u64 = 0;
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next() {
        let header = line?;
        if header.is_empty() {
            continue;
        }
        let seq = lines
            .next()
            .ok_or_else(|| anyhow!("fastq missing sequence line"))??;
        let _plus = lines
            .next()
            .ok_or_else(|| anyhow!("fastq missing plus line"))??;
        let qual = lines
            .next()
            .ok_or_else(|| anyhow!("fastq missing quality line"))??;
        reads += 1;
        let seq_bytes = seq.as_bytes();
        bases += seq_bytes.len() as u64;
        for base in seq_bytes {
            match base {
                b'G' | b'g' | b'C' | b'c' => gc += 1,
                _ => {}
            }
        }
        for q in qual.as_bytes() {
            if *q >= 33 {
                q_sum += u64::from(q - 33);
            }
        }
    }
    let mean_q = if bases > 0 {
        f64_from_u64(q_sum) / f64_from_u64(bases)
    } else {
        0.0
    };
    let gc_percent = if bases > 0 {
        (f64_from_u64(gc) / f64_from_u64(bases)) * 100.0
    } else {
        0.0
    };
    Ok(bijux_core::measure::SeqkitMetrics {
        reads,
        bases,
        mean_q,
        gc_percent,
    })
}
