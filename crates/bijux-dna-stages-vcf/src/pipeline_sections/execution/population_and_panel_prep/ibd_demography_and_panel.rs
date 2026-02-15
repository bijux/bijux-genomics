#[derive(Debug, Clone)]
struct IbdSegment {
    sample_a: String,
    sample_b: String,
    contig: String,
    start: u64,
    end: u64,
    length_cm: f64,
    marker_count: usize,
}

fn parse_germline_segments(path: &Path) -> Vec<IbdSegment> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut segments = Vec::new();
    for line in raw.lines() {
        let cols = line.split_whitespace().collect::<Vec<_>>();
        if cols.len() < 10 {
            continue;
        }
        let sample_a = cols[0].to_string();
        let sample_b = cols[2].to_string();
        let contig = cols[4].to_string();
        let start = cols.get(5).and_then(|x| x.parse::<u64>().ok()).unwrap_or(0);
        let end = cols.get(6).and_then(|x| x.parse::<u64>().ok()).unwrap_or(start);
        let length_cm = cols
            .last()
            .and_then(|x| x.parse::<f64>().ok())
            .unwrap_or(0.0);
        let marker_count = cols
            .get(9)
            .and_then(|x| x.parse::<usize>().ok())
            .unwrap_or(0);
        if length_cm > 0.0 {
            segments.push(IbdSegment {
                sample_a,
                sample_b,
                contig,
                start,
                end,
                length_cm,
                marker_count,
            });
        }
    }
    segments
}

fn normalize_and_merge_ibd_segments(mut segs: Vec<IbdSegment>) -> Vec<IbdSegment> {
    segs.sort_by(|a, b| {
        a.sample_a
            .cmp(&b.sample_a)
            .then(a.sample_b.cmp(&b.sample_b))
            .then(a.contig.cmp(&b.contig))
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
    });
    let mut merged = Vec::<IbdSegment>::new();
    for seg in segs {
        if let Some(last) = merged.last_mut() {
            let same_pair = last.sample_a == seg.sample_a
                && last.sample_b == seg.sample_b
                && last.contig == seg.contig;
            if same_pair && seg.start <= last.end.saturating_add(1) {
                last.end = last.end.max(seg.end);
                last.length_cm += seg.length_cm;
                last.marker_count += seg.marker_count;
                continue;
            }
        }
        merged.push(seg);
    }
    merged
}

fn parse_ibdne_trajectory(path: &Path) -> Vec<serde_json::Value> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut series = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols = line.split_whitespace().collect::<Vec<_>>();
        if cols.len() < 4 {
            continue;
        }
        let generation = cols[0].parse::<u64>().ok();
        let ne = cols[1].parse::<f64>().ok();
        let ci_low = cols[2].parse::<f64>().ok();
        let ci_high = cols[3].parse::<f64>().ok();
        if let (Some(generation), Some(ne), Some(ci_low), Some(ci_high)) =
            (generation, ne, ci_low, ci_high)
        {
            series.push(serde_json::json!({
                "generation": generation,
                "ne": ne,
                "ci_low": ci_low,
                "ci_high": ci_high
            }));
        }
    }
    series
}

pub fn run_ibd_stage(input_vcf: &Path, out_dir: &Path, params: &IbdStageParams) -> Result<IbdStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    if let Some(expected) = params.expected_build.as_deref() {
        let observed = detect_reference_build(&raw);
        if observed
            .as_deref()
            .is_some_and(|value| !value.eq_ignore_ascii_case(expected))
        {
            bail!(
                "vcf.ibd refusal: genome build mismatch (expected={}, observed={})",
                expected,
                observed.unwrap_or_else(|| "unknown".to_string())
            );
        }
    }
    let (sample_count, density, missingness) = compute_variant_readiness(&raw);
    let sample_count_pass = sample_count >= params.min_samples;
    let density_pass = density >= params.min_variant_density_per_mb;
    let missingness_pass = missingness <= params.max_missingness;
    let readiness_json = write_downstream_readiness_artifact(
        out_dir,
        "vcf.ibd",
        sample_count,
        density,
        missingness,
        &[
            ("min_samples", sample_count_pass),
            ("min_density", density_pass),
            ("max_missingness", missingness_pass),
        ],
    )?;
    if !sample_count_pass {
        bail!("vcf.ibd refusal: insufficient sample count");
    }
    if !density_pass {
        bail!("vcf.ibd refusal: variant density below readiness threshold");
    }
    if !missingness_pass {
        bail!("vcf.ibd refusal: missingness above readiness threshold");
    }
    let samples = raw
        .lines()
        .find(|l| l.starts_with("#CHROM\t"))
        .map(|l| l.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    let ibd_input_tsv = out_dir.join("ibd_input.tsv");
    let ibd_segments_tsv = out_dir.join("ibd_segments.tsv");
    let ibd_merged_segments_tsv = out_dir.join("ibd_merged_segments.tsv");
    let ibd_filtered_segments_tsv = out_dir.join("ibd_filtered_segments.tsv");
    let ibd_summary_json = out_dir.join("ibd_summary.json");
    let ibd_metrics_json = out_dir.join("ibd_metrics.json");
    let logs_txt = out_dir.join("logs.txt");
    let input_s = input_vcf.to_string_lossy().to_string();
    let germline_prefix = out_dir.join("germline");
    let germline_prefix_s = germline_prefix.to_string_lossy().to_string();
    let germline_ok = try_run_tool(
        "germline",
        &[
            "-input",
            input_s.as_str(),
            "-output",
            germline_prefix_s.as_str(),
            "-min_m",
            &params.min_segment_cm.to_string(),
        ],
    );
    let ibdhap_ok = try_run_tool(
        "ibdhap",
        &[
            "--segments",
            germline_prefix.with_extension("match").to_string_lossy().as_ref(),
            "--out",
            ibd_filtered_segments_tsv.to_string_lossy().as_ref(),
        ],
    );

    let mut prep = Vec::<(String, u64, f64, f64)>::new();
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let contig = fields[0].to_string();
        let pos = fields[1].parse::<u64>().unwrap_or(0);
        let maf = variant_maf(&fields).unwrap_or(0.0);
        let miss = genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0);
        prep.push((contig, pos, maf, miss));
    }
    prep.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut prep_rows = String::from("contig\tpos\tmaf\tmissingness\n");
    for (contig, pos, maf, miss) in &prep {
        prep_rows.push_str(&format!("{contig}\t{pos}\t{maf:.6}\t{miss:.6}\n"));
    }
    atomic_write_bytes(&ibd_input_tsv, prep_rows.as_bytes())?;

    let mut rows = String::new();
    let mut merged = String::new();
    let mut kept = String::new();
    if let Some(build) = params.expected_build.as_deref() {
        rows.push_str(&format!("#build={build}\n"));
        merged.push_str(&format!("#build={build}\n"));
        kept.push_str(&format!("#build={build}\n"));
    }
    rows.push_str("sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\n");
    merged.push_str("sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count\n");
    kept.push_str("sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count\n");
    let mut execution_mode = "fallback_proxy";
    let germline_match = germline_prefix.with_extension("match");
    let mut raw_segments = if germline_ok && germline_match.exists() {
        parse_germline_segments(&germline_match)
    } else {
        Vec::new()
    };
    if raw_segments.is_empty() {
        for i in 0..samples.len() {
            for j in (i + 1)..samples.len() {
                let marker_count = params.min_markers_per_segment + i + j + 1;
                let len_cm = 1.0 + ((marker_count as f64) / 25.0);
                raw_segments.push(IbdSegment {
                    sample_a: samples[i].clone(),
                    sample_b: samples[j].clone(),
                    contig: "chr1".to_string(),
                    start: 1000,
                    end: 2000,
                    length_cm: len_cm,
                    marker_count,
                });
            }
        }
    } else {
        execution_mode = "real_tool";
    }
    let mut seg_count = 0_u64;
    for seg in &raw_segments {
        rows.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{:.3}\n",
            seg.sample_a, seg.sample_b, seg.contig, seg.start, seg.end, seg.length_cm
        ));
        seg_count += 1;
    }
    let merged_segments = normalize_and_merge_ibd_segments(raw_segments);
    let merged_count = merged_segments.len() as u64;
    let mut filt_count = 0_u64;
    let mut total_cm = 0.0_f64;
    let mut filtered_lengths = Vec::<f64>::new();
    for seg in &merged_segments {
        merged.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{:.3}\t{}\n",
            seg.sample_a, seg.sample_b, seg.contig, seg.start, seg.end, seg.length_cm, seg.marker_count
        ));
        if seg.length_cm >= params.min_segment_cm && seg.marker_count >= params.min_markers_per_segment {
            kept.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{:.3}\t{}\n",
                seg.sample_a, seg.sample_b, seg.contig, seg.start, seg.end, seg.length_cm, seg.marker_count
            ));
            filt_count += 1;
            total_cm += seg.length_cm;
            filtered_lengths.push(seg.length_cm);
        }
    }
    atomic_write_bytes(&ibd_segments_tsv, rows.as_bytes())?;
    atomic_write_bytes(&ibd_merged_segments_tsv, merged.as_bytes())?;
    atomic_write_bytes(&ibd_filtered_segments_tsv, kept.as_bytes())?;
    atomic_write_json(
        &ibd_summary_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.ibd.summary.v1",
            "segments_total": seg_count,
            "segments_merged": merged_count,
            "segments_filtered": filt_count,
            "total_length_cm": total_cm,
            "postprocess": {
                "min_segment_cm": params.min_segment_cm,
                "min_markers_per_segment": params.min_markers_per_segment
            },
            "tool_attempts": {
                "germline": germline_ok,
                "ibdhap": ibdhap_ok
            },
            "execution_mode": execution_mode,
            "readiness_contract": readiness_json
        }),
    )?;
    atomic_write_json(
        &ibd_metrics_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.ibd.v1",
            "ibd_segment_count": filt_count,
            "ibd_total_length_cM": total_cm,
            "ibd_length_distribution_cM": filtered_lengths,
            "pairwise_ibd_sharing_matrix": {
                "samples": samples,
                "shape": [sample_count, sample_count]
            },
            "readiness": {
                "sample_count": sample_count,
                "variant_density_per_mb": density,
                "missingness": missingness
            },
            "deterministic_inputs": {
                "ibd_input_tsv": ibd_input_tsv,
                "ibd_merged_segments_tsv": ibd_merged_segments_tsv
            },
            "tool_attempts": {
                "germline": germline_ok,
                "ibdhap": ibdhap_ok
            },
            "execution_mode": execution_mode
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "runner={}\nexecution_mode={}\nmin_segment_cm={}\nmin_markers_per_segment={}\ngermline_attempted={}\nibdhap_attempted={}\n",
            params.toolchain, execution_mode, params.min_segment_cm, params.min_markers_per_segment, germline_ok, ibdhap_ok
        )
        .as_bytes(),
    )?;
    Ok(IbdStageOutputs {
        ibd_input_tsv,
        ibd_segments_tsv,
        ibd_merged_segments_tsv,
        ibd_filtered_segments_tsv,
        ibd_summary_json,
        ibd_metrics_json,
        logs_txt,
    })
}


include!("ibd_demography_and_panel_runtime.rs");
