use super::*;

/// # Errors
/// Returns an error if the governed remove-chimeras report JSON cannot be parsed.
pub fn parse_remove_chimeras_report(report_json: &str) -> Result<RemoveChimerasReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_remove_chimeras_report(report_json))
        .context("parse remove chimeras report")
}


/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_deduplicate_report(report_json: &str) -> Result<(u64, u64)> {
    let report = parse_remove_duplicates_report(report_json)?;
    Ok((report.reads_in, report.reads_out))
}

/// # Errors
/// Returns an error if the governed remove-duplicates report JSON cannot be parsed.
pub fn parse_remove_duplicates_report(report_json: &str) -> Result<RemoveDuplicatesReportV1> {
    match serde_json::from_str(report_json) {
        Ok(report) => Ok(report),
        Err(governed_error) => {
            if looks_like_governed_remove_duplicates_report(report_json) {
                Err(governed_error).context("parse remove duplicates report")
            } else {
                parse_legacy_remove_duplicates_report(report_json)
                    .context("parse remove duplicates report")
            }
        }
    }
}

/// # Errors
/// Returns an error if the governed remove-duplicates provenance JSON cannot be parsed.
pub fn parse_remove_duplicates_provenance(
    provenance_json: &str,
) -> Result<RemoveDuplicatesProvenanceV1> {
    serde_json::from_str(provenance_json).context("parse remove duplicates provenance")
}

/// # Errors
/// Returns an error if the duplicate-classes TSV cannot be parsed.
pub fn parse_duplicate_classes_tsv(classes_tsv: &str) -> Result<Vec<DuplicateClassEntryV1>> {
    let mut entries = Vec::new();
    for (idx, line) in classes_tsv.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if idx == 0 && line.eq_ignore_ascii_case("class\treads_removed\tpaired_mode") {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 3 {
            return Err(anyhow!(
                "duplicate classes line {} has {} columns; expected 3",
                idx + 1,
                parts.len()
            ));
        }
        entries.push(DuplicateClassEntryV1 {
            class: parts[0].trim().to_string(),
            reads_removed: parts[1].trim().parse::<u64>().with_context(|| {
                format!("parse duplicate classes line {} reads_removed", idx + 1)
            })?,
            paired_mode: serde_json::from_value(serde_json::json!(parts[2].trim()))
                .with_context(|| format!("parse duplicate classes line {} paired_mode", idx + 1))?,
        });
    }
    Ok(entries)
}

fn parse_legacy_remove_duplicates_report(report_json: &str) -> Result<RemoveDuplicatesReportV1> {
    let reads_in = parse_report_u64_field(report_json, "reads_in")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_in"))?;
    let reads_out = parse_report_u64_field(report_json, "reads_out")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_out"))?;
    let duplicates_removed = parse_report_u64_field(report_json, "duplicates_removed")
        .unwrap_or_else(|| reads_in.saturating_sub(reads_out));
    Ok(RemoveDuplicatesReportV1 {
        schema_version: "bijux.fastq.remove_duplicates.report.v1_legacy".to_string(),
        stage: "fastq.remove_duplicates".to_string(),
        stage_id: "fastq.remove_duplicates".to_string(),
        tool_id: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| {
                value
                    .get("tool_id")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string)
            })
            .unwrap_or_else(|| "unknown".to_string()),
        paired_mode: match parse_report_u64_field(report_json, "pairs_in") {
            Some(_) => crate::PairedMode::PairedEnd,
            None => crate::PairedMode::SingleEnd,
        },
        threads: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| value.get("threads").and_then(serde_json::Value::as_u64))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        dedup_mode: crate::params::remove_duplicates::DedupMode::Exact,
        keep_order: true,
        input_r1: String::new(),
        input_r2: None,
        output_r1: String::new(),
        output_r2: None,
        reads_in,
        reads_out,
        reads_in_r2: parse_report_u64_field(report_json, "reads_in_r2"),
        reads_out_r2: parse_report_u64_field(report_json, "reads_out_r2"),
        pairs_in: parse_report_u64_field(report_json, "pairs_in"),
        pairs_out: parse_report_u64_field(report_json, "pairs_out"),
        pair_count_match: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| {
                value
                    .get("pair_count_match")
                    .and_then(serde_json::Value::as_bool)
            }),
        duplicates_removed,
        dedup_rate: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| value.get("dedup_rate").and_then(serde_json::Value::as_f64))
            .unwrap_or_else(|| {
                if reads_in == 0 {
                    0.0
                } else {
                    u64_to_f64(duplicates_removed) / u64_to_f64(reads_in)
                }
            }),
        duplicate_classes_tsv: None,
        duplicate_provenance_json: None,
        duplicate_classes: Vec::new(),
        raw_backend_report: None,
        raw_backend_report_format: None,
        runtime_s: None,
        memory_mb: None,
    })
}

fn looks_like_governed_remove_duplicates_report(report_json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(report_json)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .is_some_and(|object| {
            object
                .get("schema_version")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|schema| schema.starts_with("bijux.fastq.remove_duplicates.report."))
                || object
                    .get("stage")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|stage| stage == "fastq.remove_duplicates")
                || object
                    .get("stage_id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|stage_id| stage_id == "fastq.remove_duplicates")
        })
}

fn parse_legacy_remove_chimeras_report(report_json: &str) -> Result<RemoveChimerasReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy remove chimeras json")?;
    Ok(RemoveChimerasReportV1 {
        schema_version: "bijux.fastq.remove_chimeras.report.v1_legacy".to_string(),
        stage: "fastq.remove_chimeras".to_string(),
        stage_id: "fastq.remove_chimeras".to_string(),
        tool_id: json
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        paired_mode: crate::PairedMode::SingleEnd,
        threads: json
            .get("threads")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        method: "legacy_metrics_only".to_string(),
        detection_scope: "denovo".to_string(),
        chimera_removed_definition:
            "reads flagged as chimeric are excluded from downstream abundance tables".to_string(),
        input_reads: String::new(),
        output_reads: String::new(),
        chimera_metrics_json: String::new(),
        chimeras_fasta: None,
        uchime_report_tsv: None,
        reads_in: None,
        reads_out: json
            .get("non_chimera_reads")
            .and_then(serde_json::Value::as_u64),
        chimeras_removed: json
            .get("chimeras_removed")
            .and_then(serde_json::Value::as_u64),
        chimera_fraction: json
            .get("chimera_fraction")
            .and_then(serde_json::Value::as_f64),
        used_fallback: json
            .get("used_fallback")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        raw_backend_report: None,
        raw_backend_report_format: None,
        runtime_s: None,
        memory_mb: None,
        exit_code: None,
        backend_metrics: None,
    })
}
