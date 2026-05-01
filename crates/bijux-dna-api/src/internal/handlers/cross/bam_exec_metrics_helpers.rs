fn parse_flagstat_mapped_fraction(path: &Path) -> Result<Option<f64>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut total: Option<f64> = None;
    let mut mapped: Option<f64> = None;
    for line in raw.lines() {
        let line = line.trim();
        if total.is_none() && line.contains("in total") {
            if let Some(first) = line.split_whitespace().next() {
                total = first.parse::<f64>().ok();
            }
        }
        if mapped.is_none() && line.contains(" mapped (") {
            if let Some(first) = line.split_whitespace().next() {
                mapped = first.parse::<f64>().ok();
            }
        }
    }
    let Some(total) = total else {
        return Ok(None);
    };
    let Some(mapped) = mapped else {
        return Ok(None);
    };
    if total <= 0.0 {
        return Ok(None);
    }
    Ok(Some(mapped / total))
}

fn parse_flagstat_counts(path: &Path) -> Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut total: Option<u64> = None;
    let mut mapped: Option<u64> = None;
    let mut duplicates: Option<u64> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if total.is_none() && trimmed.contains("in total") {
            total = trimmed
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        }
        if mapped.is_none() && trimmed.contains(" mapped (") {
            mapped = trimmed
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        }
        if duplicates.is_none() && trimmed.contains(" duplicates") {
            duplicates = trimmed
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        }
    }
    Ok(serde_json::json!({
        "total_reads": total,
        "mapped_reads": mapped,
        "duplicate_reads": duplicates,
        "mapped_fraction": match (total, mapped) {
            (Some(t), Some(m)) if t > 0 => {
                let mapped_f = m.to_string().parse::<f64>().ok();
                let total_f = t.to_string().parse::<f64>().ok();
                match (mapped_f, total_f) {
                    (Some(mapped_reads), Some(total_reads)) => Some(mapped_reads / total_reads),
                    _ => None,
                }
            }
            _ => None
        }
    }))
}

fn parse_mean_depth_from_depth_file(path: &Path) -> Result<Option<f64>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut n: u64 = 0;
    let mut sum: f64 = 0.0;
    for line in raw.lines() {
        let mut cols = line.split('\t');
        let _chrom = cols.next();
        let _pos = cols.next();
        if let Some(depth) = cols.next().and_then(|x| x.parse::<f64>().ok()) {
            n = n.saturating_add(1);
            sum += depth;
        }
    }
    if n == 0 {
        return Ok(None);
    }
    let n_f = n.to_string().parse::<f64>().ok();
    Ok(n_f.map(|count| sum / count))
}

fn parse_mapq_summary(path: &Path) -> Result<Option<bijux_dna_domain_bam::metrics::MapqSummaryV1>> {
    if !path.exists() {
        return Ok(None);
    }
    let (_fragment, mapq) = bam_metrics::parse_samtools_stats(path)?;
    Ok(Some(mapq))
}

fn write_bam_qc_aggregator_tsv(bam_root: &Path) -> Result<()> {
    if !bam_root.exists() {
        return Ok(());
    }
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    for entry in
        std::fs::read_dir(bam_root).with_context(|| format!("read {}", bam_root.display()))?
    {
        let entry = entry?;
        let stage_dir = entry.path();
        if !stage_dir.is_dir() {
            continue;
        }
        let stage = entry.file_name().to_string_lossy().to_string();
        let mapq_mean = parse_mapq_summary(&stage_dir.join("samtools_stats.txt"))?
            .map_or_else(|| "na".to_string(), |m| format!("{mean:.4}", mean = m.mean));
        let mapped_fraction = parse_flagstat_mapped_fraction(&stage_dir.join("flagstat.txt"))?
            .map_or_else(|| "na".to_string(), |v| format!("{v:.6}"));
        let mean_depth = parse_mean_depth_from_depth_file(&stage_dir.join("coverage.depth.txt"))?
            .map_or_else(|| "na".to_string(), |v| format!("{v:.6}"));
        let contamination = if stage_dir.join("contamination.summary.json").exists() {
            match bam_metrics::parse_contamination_json(
                &stage_dir.join("contamination.summary.json"),
            ) {
                Ok(c) => format!("{:.6}", c.estimate),
                Err(_) => "na".to_string(),
            }
        } else {
            "na".to_string()
        };
        rows.push((stage, mapped_fraction, mapq_mean, mean_depth, contamination));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut body =
        String::from("stage\tmapped_fraction\tmapq_mean\tmean_depth\tcontamination_estimate\n");
    for (stage, mapped_fraction, mapq_mean, mean_depth, contamination_estimate) in rows {
        use std::fmt::Write as _;
        let _ = writeln!(
            body,
            "{stage}\t{mapped_fraction}\t{mapq_mean}\t{mean_depth}\t{contamination_estimate}"
        );
    }
    let out = bam_root.join("bam_qc.tsv");
    bijux_dna_infra::atomic_write_bytes(&out, body.as_bytes())
        .with_context(|| format!("write {}", out.display()))
}
