fn write_downstream_readiness_artifact(
    out_dir: &Path,
    stage_id: &str,
    sample_count: usize,
    variant_density_per_mb: f64,
    missingness: f64,
    checks: &[(&str, bool)],
) -> Result<PathBuf> {
    let path = out_dir.join("vcf_ready_for_downstream.json");
    let mut check_obj = serde_json::Map::new();
    let mut all_passed = true;
    for (name, passed) in checks {
        check_obj.insert((*name).to_string(), serde_json::Value::Bool(*passed));
        all_passed &= *passed;
    }
    atomic_write_json(
        &path,
        &serde_json::json!({
            "schema_version": "bijux.vcf.ready_for_downstream.v1",
            "stage_id": stage_id,
            "sample_count": sample_count,
            "variant_density_per_mb": variant_density_per_mb,
            "missingness": missingness,
            "checks": check_obj,
            "ready_for_roh": all_passed,
            "ready_for_ibd": all_passed,
            "ready_for_demography": all_passed
        }),
    )?;
    Ok(path)
}

fn require_readiness_gate(path: &Path, field: &str, stage_id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&raw)?;
    let passes = json.get(field).and_then(|v| v.as_bool()).unwrap_or(false);
    if !passes {
        bail!(
            "{stage_id} refusal: downstream readiness gate failed ({field}=false in {})",
            path.display()
        );
    }
    Ok(())
}

pub fn run_roh_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &RohStageParams,
) -> Result<RohStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut sample_ids = Vec::<String>::new();
    let mut variants = Vec::<(String, u64, Option<f64>)>::new();
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            sample_ids = line.split('\t').skip(9).map(str::to_string).collect();
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let pos = fields[1].parse::<u64>().unwrap_or(0);
        variants.push((fields[0].to_string(), pos, variant_maf(&fields)));
    }
    if variants.is_empty() {
        bail!("vcf.roh refusal: no variants available for ROH detection");
    }
    let contig_span = {
        let min = variants.iter().map(|(_, p, _)| *p).min().unwrap_or(1);
        let max = variants.iter().map(|(_, p, _)| *p).max().unwrap_or(min + 1);
        max.saturating_sub(min).max(1)
    };
    let density = (variants.len() as f64) / ((contig_span as f64) / 1_000_000.0);
    let (_, _, missingness) = compute_variant_readiness(&raw);
    let density_pass = density >= params.min_snp_density_per_mb;
    let missingness_pass = missingness <= params.max_missingness;
    let lowcov_pass =
        missingness <= params.low_coverage_missingness_threshold || params.allow_pseudohaploid_low_coverage;
    let readiness_json = write_downstream_readiness_artifact(
        out_dir,
        "vcf.roh",
        sample_ids.len(),
        density,
        missingness,
        &[
            ("min_density", density_pass),
            ("max_missingness", missingness_pass),
            ("low_coverage_policy", lowcov_pass),
        ],
    )?;
    if !density_pass {
        bail!("vcf.roh refusal: SNP density below configured threshold");
    }
    if !missingness_pass {
        bail!("vcf.roh refusal: missingness above readiness threshold");
    }
    if !lowcov_pass {
        bail!("vcf.roh refusal: low-coverage regime requires explicit pseudo-haploid support");
    }

    let roh_segments_tsv = out_dir.join("roh_segments.tsv");
    let roh_per_sample_tsv = out_dir.join("roh_per_sample.tsv");
    let roh_json = out_dir.join("roh.json");
    let metrics_json = out_dir.join("metrics.json");
    let roh_summary_json = out_dir.join("roh_summary.json");
    let roh_metrics_json = out_dir.join("roh_metrics.json");
    let logs_txt = out_dir.join("logs.txt");
    let plink_prefix = out_dir.join("roh_plink");
    let plink_prefix_s = plink_prefix.to_string_lossy().to_string();
    let input_s = input_vcf.to_string_lossy().to_string();
    let plink_homozyg_ok = try_run_tool(
        "plink2",
        &[
            "--vcf",
            input_s.as_str(),
            "--double-id",
            "--allow-extra-chr",
            "--homozyg",
            "--homozyg-window-snp",
            &params.plink_homozyg_window_snp.to_string(),
            "--homozyg-kb",
            &params.plink_homozyg_kb.to_string(),
            "--homozyg-gap",
            &params.plink_homozyg_gap_kb.to_string(),
            "--out",
            plink_prefix_s.as_str(),
        ],
    );

    let mut rows = String::from("sample\tcontig\tstart\tend\tlength_bp\tn_sites\n");
    let mut total_length = 0_u64;
    let mut segment_lengths = Vec::<u64>::new();
    for sample in &sample_ids {
        let mut start = variants[0].1;
        let mut last = variants[0].1;
        let contig = variants[0].0.clone();
        let mut n_sites = 1_u64;
        for (chr, pos, maf) in variants.iter().skip(1) {
            if *chr != contig || pos.saturating_sub(last) > params.max_gap_bp || maf.unwrap_or(0.5) < 0.001 {
                let len = last.saturating_sub(start).max(1);
                if len >= params.min_segment_kb * 1000 {
                    rows.push_str(&format!("{sample}\t{contig}\t{start}\t{last}\t{len}\t{n_sites}\n"));
                    total_length += len;
                    segment_lengths.push(len);
                }
                start = *pos;
                n_sites = 1;
            } else {
                n_sites += 1;
            }
            last = *pos;
        }
        let len = last.saturating_sub(start).max(1);
        if len >= params.min_segment_kb * 1000 {
            rows.push_str(&format!("{sample}\t{contig}\t{start}\t{last}\t{len}\t{n_sites}\n"));
            total_length += len;
            segment_lengths.push(len);
        }
    }
    atomic_write_bytes(&roh_segments_tsv, rows.as_bytes())?;
    let segment_count = segment_lengths.len() as u64;
    if segment_count > params.max_segment_count {
        bail!("vcf.roh refusal: segment count sanity check failed");
    }
    let mean_len = if segment_lengths.is_empty() {
        0.0
    } else {
        segment_lengths.iter().sum::<u64>() as f64 / segment_lengths.len() as f64
    };
    let mut per_sample_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut per_sample_lengths = std::collections::BTreeMap::<String, u64>::new();
    for line in rows.lines().skip(1) {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 6 {
            continue;
        }
        let sample = fields[0].to_string();
        let len = fields[4].parse::<u64>().unwrap_or(0);
        *per_sample_counts.entry(sample.clone()).or_insert(0) += 1;
        *per_sample_lengths.entry(sample).or_insert(0) += len;
    }
    let mut per_sample_tsv = String::from("sample\tsegment_count\ttotal_length_bp\tmean_length_bp\n");
    for sample in &sample_ids {
        let count = *per_sample_counts.get(sample).unwrap_or(&0);
        let total = *per_sample_lengths.get(sample).unwrap_or(&0);
        let mean = if count == 0 { 0.0 } else { total as f64 / count as f64 };
        per_sample_tsv.push_str(&format!("{sample}\t{count}\t{total}\t{mean:.3}\n"));
    }
    atomic_write_bytes(&roh_per_sample_tsv, per_sample_tsv.as_bytes())?;
    let roh_payload = serde_json::json!({
        "schema_version": "bijux.vcf.roh.summary.v2",
        "toolchain": params.toolchain,
        "segment_count": segment_count,
        "total_length_bp": total_length,
        "mean_length_bp": mean_len,
        "per_sample_summary_tsv": roh_per_sample_tsv,
        "readiness": {
            "variant_density_per_mb": density,
            "missingness": missingness,
            "low_coverage_missingness_threshold": params.low_coverage_missingness_threshold,
            "allow_pseudohaploid_low_coverage": params.allow_pseudohaploid_low_coverage
        },
        "tool_attempts": {
            "plink2_homozyg": plink_homozyg_ok
        }
    });
    atomic_write_json(&roh_json, &roh_payload)?;
    atomic_write_json(
        &roh_summary_json,
        &roh_payload,
    )?;
    let metrics_payload = serde_json::json!({
        "schema_version": "bijux.vcf.metrics.v1",
        "roh": {
            "roh_count": segment_count,
            "roh_total_mb": total_length as f64 / 1_000_000.0,
            "roh_mean_length_mb": mean_len / 1_000_000.0,
            "distribution_bp": segment_lengths,
            "readiness": {
                "variant_density_per_mb": density,
                "missingness": missingness
            },
            "tool_attempts": {
                "plink2_homozyg": plink_homozyg_ok
            },
            "readiness_contract": readiness_json
        }
    });
    atomic_write_json(&metrics_json, &metrics_payload)?;
    atomic_write_json(
        &roh_metrics_json,
        &metrics_payload,
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "runner=plink2_homozyg_like\ntoolchain={}\nmin_snp_density_per_mb={}\nmax_missingness={}\nmin_segment_kb={}\nmax_gap_bp={}\nmax_segment_count={}\nplink2_homozyg_attempted={}\n",
            params.toolchain,
            params.min_snp_density_per_mb,
            params.max_missingness,
            params.min_segment_kb,
            params.max_gap_bp,
            params.max_segment_count,
            plink_homozyg_ok
        )
        .as_bytes(),
    )?;

    Ok(RohStageOutputs {
        roh_segments_tsv,
        roh_per_sample_tsv,
        roh_json,
        metrics_json,
        roh_summary_json,
        roh_metrics_json,
        logs_txt,
    })
}

fn compute_variant_readiness(raw: &str) -> (usize, f64, f64) {
    let mut samples = 0usize;
    let mut min_pos = u64::MAX;
    let mut max_pos = 0_u64;
    let mut variants = 0_u64;
    let mut missing = 0_u64;
    let mut total_gt = 0_u64;
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            samples = line.split('\t').skip(9).count();
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        variants += 1;
        let pos = fields[1].parse::<u64>().unwrap_or(0);
        min_pos = min_pos.min(pos);
        max_pos = max_pos.max(pos);
        if fields.len() > 9 {
            let gt_idx = parse_format_index(&fields, "GT");
            if let Some(idx) = gt_idx {
                for sample in &fields[9..] {
                    let vals = sample.split(':').collect::<Vec<_>>();
                    if let Some(gt) = vals.get(idx) {
                        total_gt += 1;
                        if gt.contains('.') {
                            missing += 1;
                        }
                    }
                }
            }
        }
    }
    let span = max_pos.saturating_sub(min_pos).max(1);
    let density = variants as f64 / (span as f64 / 1_000_000.0);
    let miss = if total_gt == 0 {
        0.0
    } else {
        missing as f64 / total_gt as f64
    };
    (samples, density, miss)
}

fn detect_reference_build(raw: &str) -> Option<String> {
    raw.lines()
        .find_map(|line| line.strip_prefix("##reference=").map(str::trim))
        .map(ToString::to_string)
}

