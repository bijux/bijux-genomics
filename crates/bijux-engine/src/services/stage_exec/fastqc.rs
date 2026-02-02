fn fastqc_metrics_v2_from_dir(dir: &Path) -> Option<FastqcMetricsV2> {
    let path = find_fastqc_data(dir)?;
    let raw = std::fs::read_to_string(path).ok()?;
    let modules = parse_fastqc_modules(&raw);

    let per_base_quality = modules
        .get("Per base sequence quality")
        .and_then(|lines| parse_per_base_quality(lines));
    let gc_distribution = modules
        .get("Per sequence GC content")
        .and_then(|lines| parse_gc_distribution(lines));
    let adapter_content = modules
        .get("Adapter Content")
        .and_then(|lines| parse_adapter_content(lines));
    let duplication = modules
        .get("Sequence Duplication Levels")
        .map(|lines| parse_duplication(lines));
    let n_content = modules
        .get("Per base N content")
        .and_then(|lines| parse_n_content(lines));
    let kmer_content = modules
        .get("Kmer Content")
        .map(|lines| parse_kmer_content(lines));

    Some(FastqcMetricsV2 {
        schema_version: "bijux.fastqc_metrics.v2".to_string(),
        source: dir.display().to_string(),
        per_base_quality,
        gc_distribution,
        adapter_content,
        duplication,
        n_content,
        kmer_content,
    })
}

fn parse_per_base_quality(lines: &[String]) -> Option<PerBaseQualitySummary> {
    let mut means = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let mean = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let Some(mean) = mean {
            means.push(mean);
        }
    }
    if means.is_empty() {
        return None;
    }
    let mean_min = means.iter().copied().fold(f64::INFINITY, f64::min);
    let mean_max = means.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    #[allow(clippy::cast_precision_loss)]
    let mean_mean = means.iter().sum::<f64>() / means.len() as f64;
    let bases_below_q20 = means.iter().filter(|v| **v < 20.0).count() as u64;
    let bases_below_q30 = means.iter().filter(|v| **v < 30.0).count() as u64;
    Some(PerBaseQualitySummary {
        mean_min,
        mean_max,
        mean_mean,
        bases_below_q20,
        bases_below_q30,
    })
}

fn parse_gc_distribution(lines: &[String]) -> Option<GcDistributionSummary> {
    let mut total = 0.0;
    let mut weighted_sum = 0.0;
    let mut counts = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let gc = parts.first().and_then(|v| v.parse::<f64>().ok());
        let count = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let (Some(gc), Some(count)) = (gc, count) {
            total += count;
            weighted_sum += gc * count;
            counts.push(count);
        }
    }
    if total <= 0.0 {
        return None;
    }
    let mean_gc = weighted_sum / total;
    let mut var_sum = 0.0;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let gc = parts.first().and_then(|v| v.parse::<f64>().ok());
        let count = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let (Some(gc), Some(count)) = (gc, count) {
            var_sum += (gc - mean_gc).powi(2) * count;
        }
    }
    let std_gc = (var_sum / total).sqrt();
    #[allow(clippy::cast_precision_loss)]
    let mean_count = counts.iter().sum::<f64>() / counts.len() as f64;
    let mut count_var = 0.0;
    for count in &counts {
        count_var += (count - mean_count).powi(2);
    }
    #[allow(clippy::cast_precision_loss)]
    let count_std = (count_var / counts.len() as f64).sqrt();
    let outlier = counts
        .iter()
        .any(|count| *count > mean_count + (3.0 * count_std));
    Some(GcDistributionSummary {
        mean_gc,
        std_gc,
        outlier,
    })
}

fn parse_adapter_content(lines: &[String]) -> Option<AdapterContentSummary> {
    let mut header: Option<Vec<String>> = None;
    let mut per_adapter: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }
        if header.is_none() && parts[0].to_lowercase().contains("position") {
            header = Some(parts.iter().map(std::string::ToString::to_string).collect());
            continue;
        }
        let Some(header) = header.as_ref() else {
            continue;
        };
        if parts.len() < header.len() {
            continue;
        }
        for (idx, name) in header.iter().enumerate().skip(1) {
            let value = parts
                .get(idx)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            per_adapter.entry(name.clone()).or_default().push(value);
        }
    }
    if per_adapter.is_empty() {
        return None;
    }
    let mut adapters = Vec::new();
    let mut max_percent: f64 = 0.0;
    let mut sum = 0.0;
    let mut count = 0.0;
    for (name, values) in &per_adapter {
        if values.is_empty() {
            continue;
        }
        let local_max = values.iter().copied().fold(0.0, f64::max);
        #[allow(clippy::cast_precision_loss)]
        let local_mean = values.iter().sum::<f64>() / values.len() as f64;
        max_percent = max_percent.max(local_max);
        sum += values.iter().sum::<f64>();
        #[allow(clippy::cast_precision_loss)]
        {
            count += values.len() as f64;
        }
        adapters.push(AdapterSignal {
            name: name.clone(),
            max_percent: local_max,
            mean_percent: local_mean,
        });
    }
    let mean_percent = if count > 0.0 { sum / count } else { 0.0 };
    Some(AdapterContentSummary {
        max_percent,
        mean_percent,
        adapters,
    })
}

fn parse_duplication(lines: &[String]) -> DuplicationSummary {
    let mut unique_fraction = None;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let level = parts.first().and_then(|v| v.parse::<u64>().ok());
        let percent = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if level == Some(1) {
            unique_fraction = percent.map(|v| v / 100.0);
            break;
        }
    }
    let unique_fraction = unique_fraction.unwrap_or(0.0);
    DuplicationSummary {
        unique_fraction,
        duplication_rate: (1.0 - unique_fraction).max(0.0),
    }
}

fn parse_n_content(lines: &[String]) -> Option<NContentSummary> {
    let mut values = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        if let Some(value) = parts.get(1).and_then(|v| v.parse::<f64>().ok()) {
            values.push(value);
        }
    }
    if values.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let mean_percent = values.iter().sum::<f64>() / values.len() as f64;
    let max_percent = values.iter().copied().fold(0.0, f64::max);
    Some(NContentSummary {
        mean_percent,
        max_percent,
    })
}

fn parse_kmer_content(lines: &[String]) -> KmerSummary {
    let mut warning_count = 0_u64;
    let mut top_kmer: Option<(String, f64)> = None;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let seq = parts.first().copied().unwrap_or("").to_string();
        let count = parts
            .get(1)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        warning_count += 1;
        if top_kmer.as_ref().is_none_or(|(_, c)| count > *c) {
            top_kmer = Some((seq, count));
        }
    }
    KmerSummary {
        warning_count,
        top_kmer: top_kmer.map(|(seq, _)| seq),
    }
}

fn adapter_suggestions_from_fastqc(dir: &Path) -> (serde_json::Value, Option<String>) {
    let Some(path) = find_fastqc_data(dir) else {
        return (serde_json::json!({}), None);
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (serde_json::json!({}), None);
    };
    let modules = parse_fastqc_modules(&raw);
    let mut in_module = false;
    let mut candidates = Vec::new();
    for line in raw.lines() {
        if line.starts_with(">>Overrepresented sequences") {
            in_module = true;
            continue;
        }
        if in_module && line.starts_with(">>END_MODULE") {
            break;
        }
        if !in_module || line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let sequence = parts[0].to_string();
        let percent = parts[2].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
        let possible_source = parts.get(3).map(ToString::to_string);
        let (confidence, confidence_score) = confidence_from_percent(percent);
        let matched_preset = if sequence.contains("AGATCGGAAGAGC") {
            Some("illumina-default".to_string())
        } else if sequence.contains("CTGTCTCTTATA") || sequence.contains("TGGAATTCTCGG") {
            Some("ssdna".to_string())
        } else {
            None
        };
        candidates.push(serde_json::json!({
            "kind": "overrepresented",
            "sequence": sequence,
            "percent": percent,
            "source": possible_source,
            "matched_preset": matched_preset,
            "confidence": confidence,
            "confidence_score": confidence_score,
        }));
    }
    if let Some(lines) = modules.get("Adapter Content") {
        if let Some(adapter_content) = parse_adapter_content(lines) {
            for adapter in adapter_content.adapters {
                let (confidence, confidence_score) = confidence_from_percent(adapter.max_percent);
                let matched_preset = if adapter.name.to_lowercase().contains("illumina") {
                    Some("illumina-default".to_string())
                } else {
                    None
                };
                candidates.push(serde_json::json!({
                    "kind": "adapter_content",
                    "adapter_name": adapter.name,
                    "max_percent": adapter.max_percent,
                    "mean_percent": adapter.mean_percent,
                    "matched_preset": matched_preset,
                    "confidence": confidence,
                    "confidence_score": confidence_score,
                }));
            }
        }
    }
    let suggested_preset = candidates
        .iter()
        .find_map(|entry| entry.get("matched_preset").and_then(|v| v.as_str()))
        .map(str::to_string);
    (
        serde_json::json!({
            "schema_version": "bijux.adapter_suggestions.v1",
            "candidates": candidates,
            "suggested_preset": suggested_preset,
        }),
        suggested_preset,
    )
}

fn confidence_from_percent(percent: f64) -> (&'static str, f64) {
    if percent >= 1.0 {
        ("high", 0.9)
    } else if percent >= 0.1 {
        ("medium", 0.6)
    } else if percent >= 0.01 {
        ("low", 0.3)
    } else {
        ("low", 0.1)
    }
}

fn parse_screen_report(path: &Path) -> Result<(f64, serde_json::Value)> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("screen report missing: {}", path.display()))?;
    let mut entries = Vec::new();
    let mut unmapped_percent = None;
    let mut errors = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            errors.push(format!("line {} has {} columns", idx + 1, parts.len()));
            continue;
        }
        let label = parts[0].trim().to_string();
        let percent_col = parts
            .last()
            .ok_or_else(|| anyhow!("screen report line {} missing percent", idx + 1))?;
        let percent_str = percent_col.trim().trim_end_matches('%');
        let percent = percent_str
            .parse::<f64>()
            .with_context(|| format!("screen report line {} percent parse", idx + 1))?;
        let label_lower = label.to_lowercase();
        if label_lower.contains("unmapped")
            || (label_lower.contains("no hit") && unmapped_percent.is_none())
        {
            unmapped_percent = Some(percent);
        }
        entries.push(serde_json::json!({
            "reference": label,
            "percent": percent,
        }));
    }
    if !errors.is_empty() {
        return Err(anyhow!("screen report parse errors: {}", errors.join("; ")));
    }
    if entries.is_empty() {
        return Ok((
            0.0,
            serde_json::json!({
                "schema_version": "bijux.screen_summary.v1",
                "entries": entries,
                "warning": "empty_report",
            }),
        ));
    }
    let contamination_rate = unmapped_percent.map_or(0.0, |value| (100.0 - value).max(0.0) / 100.0);
    Ok((
        contamination_rate,
        serde_json::json!({
            "schema_version": "bijux.screen_summary.v1",
            "entries": entries,
        }),
    ))
}

#[cfg(test)]
mod screen_tests {
    use super::parse_screen_report;
    use anyhow::Result;
    use std::fs;

    #[test]
    fn parse_screen_report_parses_fixture() -> Result<()> {
        let fixture = include_str!("../../../tests/fixtures/screen/screen_report_v1.tsv");
        let dir = std::env::temp_dir().join("bijux-screen-fixture");
        fs::create_dir_all(&dir)?;
        let path = dir.join("screen_report.tsv");
        fs::write(&path, fixture)?;
        let (rate, summary) = parse_screen_report(&path)?;
        assert!((rate - 0.02).abs() < 1e-6);
        assert!(summary.get("entries").is_some());
        Ok(())
    }

    #[test]
    fn parse_screen_report_rejects_bad_fixture() {
        let fixture = include_str!("../../../tests/fixtures/screen/screen_report_bad.tsv");
        let dir = std::env::temp_dir().join("bijux-screen-fixture-bad");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("screen_report.tsv");
        let _ = fs::write(&path, fixture);
        let result = parse_screen_report(&path);
        assert!(result.is_err());
    }
}

fn tool_supports_polyx(tool_id: &str) -> bool {
    matches!(tool_id, "fastp")
}

fn tool_supports_kmer_filter(tool_id: &str) -> bool {
    matches!(tool_id, "bbduk")
}

fn polyx_unsupported_warning(tool_id: &str, params: &serde_json::Value) -> Option<String> {
    if params.get("polyx_bank").is_some() && !tool_supports_polyx(tool_id) {
        return Some(format!(
            "warning: polyx preset requested but tool '{tool_id}' does not advertise polyX support"
        ));
    }
    None
}
