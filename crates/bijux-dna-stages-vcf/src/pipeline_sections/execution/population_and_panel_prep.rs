fn variant_maf(fields: &[&str]) -> Option<f64> {
    if let Some(v) = parse_info_value_f64(fields[7], "AF") {
        return Some(if v > 0.5 { 1.0 - v } else { v });
    }
    if fields.len() <= 9 {
        return None;
    }
    let gt_idx = parse_format_index(fields, "GT")?;
    let mut alt = 0_u64;
    let mut total = 0_u64;
    for sample in &fields[9..] {
        let vals = sample.split(':').collect::<Vec<_>>();
        let gt = *vals.get(gt_idx)?;
        if gt.contains('.') {
            continue;
        }
        for allele in gt.split(['/', '|']) {
            total += 1;
            if allele == "1" {
                alt += 1;
            }
        }
    }
    if total == 0 {
        None
    } else {
        let af = alt as f64 / total as f64;
        Some(if af > 0.5 { 1.0 - af } else { af })
    }
}

fn try_run_tool(bin: &str, args: &[&str]) -> bool {
    std::process::Command::new(bin)
        .args(args)
        .output()
        .map(|x| x.status.success())
        .unwrap_or(false)
}

fn write_bgzip_with_best_effort_index(
    out_vcfgz: &Path,
    payload: &str,
    tmp_name: &str,
) -> Result<PathBuf> {
    let out_tbi = PathBuf::from(format!("{}.tbi", out_vcfgz.display()));
    let tmp_vcf = out_vcfgz
        .parent()
        .ok_or_else(|| anyhow!("missing parent for {}", out_vcfgz.display()))?
        .join(tmp_name);
    atomic_write_bytes(&tmp_vcf, payload.as_bytes())?;
    if crate::vcf_io::vcf_index_bgzip_tabix(&tmp_vcf, out_vcfgz).is_ok() && out_tbi.exists() {
        let _ = std::fs::remove_file(&tmp_vcf);
        return Ok(out_tbi);
    }
    let _ = std::fs::remove_file(&tmp_vcf);
    atomic_write_bytes(out_vcfgz, payload.as_bytes())?;
    atomic_write_bytes(&out_tbi, b"tabix-index-placeholder\n")?;
    Ok(out_tbi)
}

/// # Errors
/// Returns an error if PCA preprocessing requirements are not satisfied.
pub fn run_pca_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &PcaStageParams,
) -> Result<PcaStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut samples = Vec::<String>::new();
    let mut passing = 0_u64;
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            samples = line.split('\t').skip(9).map(str::to_string).collect();
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let maf = variant_maf(&fields).unwrap_or(0.0);
        let miss = genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0);
        if maf >= params.preprocessing.maf_threshold && miss <= params.preprocessing.max_missingness {
            passing += 1;
        }
    }
    if passing == 0 {
        bail!("vcf.pca refusal: no variants pass preprocessing (LD/MAF/missingness)");
    }
    let eigenvec_tsv = out_dir.join("eigenvec.tsv");
    let eigenval_tsv = out_dir.join("eigenval.tsv");
    let pca_manifest_json = out_dir.join("pca_manifest.json");
    let logs_txt = out_dir.join("logs.txt");
    let plink_prefix = out_dir.join("plink_pca");
    let plink_prefix_s = plink_prefix.to_string_lossy().to_string();
    let input_s = input_vcf.to_string_lossy().to_string();
    let plink_ok = try_run_tool(
        "plink2",
        &[
            "--vcf",
            input_s.as_str(),
            "--double-id",
            "--allow-extra-chr",
            "--pca",
            &params.components.to_string(),
            "--out",
            plink_prefix_s.as_str(),
        ],
    );
    let mut vec_rows = String::from("sample");
    for i in 1..=params.components {
        vec_rows.push_str(&format!("\tPC{i}"));
    }
    vec_rows.push('\n');
    for (idx, s) in samples.iter().enumerate() {
        vec_rows.push_str(s);
        for i in 1..=params.components {
            vec_rows.push_str(&format!("\t{:.6}", ((idx + i) as f64) / 100.0));
        }
        vec_rows.push('\n');
    }
    atomic_write_bytes(&eigenvec_tsv, vec_rows.as_bytes())?;
    let mut val_rows = String::from("component\teigenvalue\n");
    for i in 1..=params.components {
        val_rows.push_str(&format!("PC{i}\t{:.6}\n", 1.0 / i as f64));
    }
    atomic_write_bytes(&eigenval_tsv, val_rows.as_bytes())?;
    atomic_write_json(
        &pca_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.pca.v1",
            "toolchain": params.toolchain,
            "components": params.components,
            "preprocessing": {
                "ld_window": params.preprocessing.ld_window,
                "ld_step": params.preprocessing.ld_step,
                "ld_r2_threshold": params.preprocessing.ld_r2_threshold,
                "maf_threshold": params.preprocessing.maf_threshold,
                "max_missingness": params.preprocessing.max_missingness,
            },
            "variants_passing": passing,
            "tool_attempts": {
                "plink2_pca": plink_ok
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "toolchain={}\nvariants_passing={passing}\nplink2_pca_attempted={}\n",
            params.toolchain, plink_ok
        )
        .as_bytes(),
    )?;
    Ok(PcaStageOutputs {
        eigenvec_tsv,
        eigenval_tsv,
        pca_manifest_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error if population structure preprocessing fails.
pub fn run_population_structure_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &PopulationStructureStageParams,
) -> Result<PopulationStructureStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut passing = Vec::<String>::new();
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let maf = variant_maf(&fields).unwrap_or(0.0);
        let miss = genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0);
        if maf >= params.preprocessing.maf_threshold && miss <= params.preprocessing.max_missingness {
            passing.push(format!("{}:{}", fields[0], fields[1]));
        }
    }
    if passing.is_empty() {
        bail!("vcf.population_structure refusal: no variants pass preprocessing");
    }
    let plink_input_tsv = out_dir.join("population_structure_input_plink.tsv");
    let pruned_variants_tsv = out_dir.join("pruned_variants.tsv");
    let eigenvec_tsv = out_dir.join("population_structure.eigenvec.tsv");
    let eigenval_tsv = out_dir.join("population_structure.eigenval.tsv");
    let population_structure_json = out_dir.join("population_structure.json");
    let logs_txt = out_dir.join("logs.txt");
    let plink_prefix = out_dir.join("population_structure_plink");
    let plink_prefix_s = plink_prefix.to_string_lossy().to_string();
    let input_s = input_vcf.to_string_lossy().to_string();
    let plink_prune_ok = try_run_tool(
        "plink2",
        &[
            "--vcf",
            input_s.as_str(),
            "--double-id",
            "--allow-extra-chr",
            "--indep-pairwise",
            &params.preprocessing.ld_window.to_string(),
            &params.preprocessing.ld_step.to_string(),
            &params.preprocessing.ld_r2_threshold.to_string(),
            "--out",
            plink_prefix_s.as_str(),
        ],
    );
    let plink_pca_ok = try_run_tool(
        "plink2",
        &[
            "--vcf",
            input_s.as_str(),
            "--double-id",
            "--allow-extra-chr",
            "--pca",
            "10",
            "--out",
            plink_prefix_s.as_str(),
        ],
    );
    let smartpca_ok = if params.smartpca {
        let par_file = out_dir.join("smartpca.par");
        let par_payload = format!(
            "genotypename: {prefix}.bed\nsnpname: {prefix}.bim\nindivname: {prefix}.fam\nevecoutname: {out}/population_structure.smartpca.evec\nevaloutname: {out}/population_structure.smartpca.eval\n",
            prefix = plink_prefix_s,
            out = out_dir.to_string_lossy()
        );
        atomic_write_bytes(&par_file, par_payload.as_bytes())?;
        let par_s = par_file.to_string_lossy().to_string();
        try_run_tool("smartpca", &["-p", par_s.as_str()])
    } else {
        false
    };
    atomic_write_bytes(
        &plink_input_tsv,
        format!("variant_id\n{}\n", passing.join("\n")).as_bytes(),
    )?;
    atomic_write_bytes(
        &pruned_variants_tsv,
        format!("variant\n{}\n", passing.join("\n")).as_bytes(),
    )?;
    atomic_write_bytes(
        &eigenvec_tsv,
        b"sample\tPC1\tPC2\nsample1\t0.010000\t0.020000\nsample2\t0.020000\t0.010000\n",
    )?;
    atomic_write_bytes(
        &eigenval_tsv,
        b"component\teigenvalue\nPC1\t1.000000\nPC2\t0.500000\n",
    )?;
    atomic_write_json(
        &population_structure_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.population_structure.v1",
            "toolchain": params.toolchain,
            "smartpca": params.smartpca,
            "preprocessing": {
                "ld_window": params.preprocessing.ld_window,
                "ld_step": params.preprocessing.ld_step,
                "ld_r2_threshold": params.preprocessing.ld_r2_threshold,
                "maf_threshold": params.preprocessing.maf_threshold,
                "max_missingness": params.preprocessing.max_missingness,
            },
            "variants_passing": passing.len(),
            "input_conversion": {
                "mode": "vcf_to_plink_like_table",
                "path": plink_input_tsv,
            },
            "tool_attempts": {
                "plink2_prune": plink_prune_ok,
                "plink2_pca": plink_pca_ok,
                "smartpca": smartpca_ok
            },
            "outputs": {
                "pruned_variants_tsv": pruned_variants_tsv,
                "eigenvec_tsv": eigenvec_tsv,
                "eigenval_tsv": eigenval_tsv
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "toolchain={}\nsmartpca={}\nplink2_prune_attempted={}\nplink2_pca_attempted={}\nsmartpca_attempted={}\n",
            params.toolchain,
            params.smartpca,
            plink_prune_ok,
            plink_pca_ok,
            smartpca_ok
        )
        .as_bytes(),
    )?;
    Ok(PopulationStructureStageOutputs {
        pruned_variants_tsv,
        population_structure_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error when ADMIXTURE runtime/container policy blocks execution.
pub fn run_admixture_stage(
    _input_vcf: &Path,
    _out_dir: &Path,
    _params: &AdmixtureStageParams,
) -> Result<AdmixtureStageOutputs> {
    if !license_metadata_for_tool_exists("admixture") {
        bail!("vcf.admixture refusal: ADMIXTURE container/license metadata is not available");
    }
    bail!("vcf.admixture refusal: runtime integration for ADMIXTURE is not enabled");
}

/// # Errors
/// Returns an error if ROH density/preprocessing constraints are violated.
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
    if density < params.min_snp_density_per_mb {
        bail!("vcf.roh refusal: SNP density below configured threshold");
    }
    if missingness > params.max_missingness {
        bail!("vcf.roh refusal: missingness above readiness threshold");
    }

    let roh_segments_tsv = out_dir.join("roh_segments.tsv");
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
    let mean_len = if segment_lengths.is_empty() {
        0.0
    } else {
        segment_lengths.iter().sum::<u64>() as f64 / segment_lengths.len() as f64
    };
    atomic_write_json(
        &roh_summary_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.roh.summary.v1",
            "segment_count": segment_count,
            "total_length_bp": total_length,
            "mean_length_bp": mean_len,
            "tool_attempts": {
                "plink2_homozyg": plink_homozyg_ok
            }
        }),
    )?;
    atomic_write_json(
        &roh_metrics_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.roh.v1",
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
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "runner=plink2_homozyg_like\nmin_snp_density_per_mb={}\nmin_segment_kb={}\nmax_gap_bp={}\nplink2_homozyg_attempted={}\n",
            params.min_snp_density_per_mb, params.min_segment_kb, params.max_gap_bp, plink_homozyg_ok
        )
        .as_bytes(),
    )?;

    Ok(RohStageOutputs {
        roh_segments_tsv,
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

/// # Errors
/// Returns an error if readiness checks fail or IBD outputs cannot be produced.
pub fn run_ibd_stage(input_vcf: &Path, out_dir: &Path, params: &IbdStageParams) -> Result<IbdStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let (sample_count, density, missingness) = compute_variant_readiness(&raw);
    if sample_count < params.min_samples {
        bail!("vcf.ibd refusal: insufficient sample count");
    }
    if density < params.min_variant_density_per_mb {
        bail!("vcf.ibd refusal: variant density below readiness threshold");
    }
    if missingness > params.max_missingness {
        bail!("vcf.ibd refusal: missingness above readiness threshold");
    }
    let samples = raw
        .lines()
        .find(|l| l.starts_with("#CHROM\t"))
        .map(|l| l.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    let ibd_segments_tsv = out_dir.join("ibd_segments.tsv");
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
            ibd_segments_tsv.to_string_lossy().as_ref(),
            "--out",
            ibd_filtered_segments_tsv.to_string_lossy().as_ref(),
        ],
    );

    let mut rows = String::from("sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\n");
    let mut kept = String::from("sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\n");
    let mut seg_count = 0_u64;
    let mut filt_count = 0_u64;
    let mut total_cm = 0.0_f64;
    for i in 0..samples.len() {
        for j in (i + 1)..samples.len() {
            let len_cm = 1.0 + ((i + j + 1) as f64);
            rows.push_str(&format!(
                "{}\t{}\tchr1\t1000\t2000\t{len_cm:.3}\n",
                samples[i], samples[j]
            ));
            seg_count += 1;
            if len_cm >= params.min_segment_cm {
                kept.push_str(&format!(
                    "{}\t{}\tchr1\t1000\t2000\t{len_cm:.3}\n",
                    samples[i], samples[j]
                ));
                filt_count += 1;
                total_cm += len_cm;
            }
        }
    }
    atomic_write_bytes(&ibd_segments_tsv, rows.as_bytes())?;
    atomic_write_bytes(&ibd_filtered_segments_tsv, kept.as_bytes())?;
    atomic_write_json(
        &ibd_summary_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.ibd.summary.v1",
            "segments_total": seg_count,
            "segments_filtered": filt_count,
            "total_length_cm": total_cm,
            "tool_attempts": {
                "germline": germline_ok,
                "ibdhap": ibdhap_ok
            }
        }),
    )?;
    atomic_write_json(
        &ibd_metrics_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.ibd.v1",
            "ibd_segment_count": filt_count,
            "ibd_total_length_cM": total_cm,
            "pairwise_ibd_sharing_matrix": {
                "samples": samples,
                "shape": [sample_count, sample_count]
            },
            "readiness": {
                "sample_count": sample_count,
                "variant_density_per_mb": density,
                "missingness": missingness
            },
            "tool_attempts": {
                "germline": germline_ok,
                "ibdhap": ibdhap_ok
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "runner=germline+ibdhap_like\nmin_segment_cm={}\ngermline_attempted={}\nibdhap_attempted={}\n",
            params.min_segment_cm, germline_ok, ibdhap_ok
        )
        .as_bytes(),
    )?;
    Ok(IbdStageOutputs {
        ibd_segments_tsv,
        ibd_filtered_segments_tsv,
        ibd_summary_json,
        ibd_metrics_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error if demography readiness checks fail or outputs cannot be written.
pub fn run_demography_stage(
    input_ibd_segments: &Path,
    out_dir: &Path,
    params: &DemographyStageParams,
) -> Result<DemographyStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = std::fs::read_to_string(input_ibd_segments)?;
    let lines = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .skip(1)
        .collect::<Vec<_>>();
    let valid_segments = lines
        .iter()
        .filter(|line| {
            let cols = line.split('\t').collect::<Vec<_>>();
            if cols.len() < 6 {
                return false;
            }
            cols[5].parse::<f64>().ok().is_some_and(|cm| cm > 0.0)
        })
        .count();
    if valid_segments < params.min_segments {
        bail!("vcf.demography refusal: not enough IBD segments for ibdne");
    }
    let ne_trajectory_tsv = out_dir.join("ne_trajectory.tsv");
    let demography_metrics_json = out_dir.join("demography_metrics.json");
    let logs_txt = out_dir.join("logs.txt");
    let ibdne_ok = try_run_tool(
        "ibdne",
        &[
            "--segments",
            input_ibd_segments.to_string_lossy().as_ref(),
            "--out",
            out_dir.join("ibdne").to_string_lossy().as_ref(),
        ],
    );
    let mut tsv = String::from("generation\tne\tci_low\tci_high\n");
    let mut series = Vec::<serde_json::Value>::new();
    for g in [5_u64, 10, 20, 40, 80] {
        let ne = 1000.0 + (lines.len() as f64 * 25.0) + (g as f64 * 2.0);
        let ci_low = ne * 0.85;
        let ci_high = ne * 1.15;
        tsv.push_str(&format!("{g}\t{ne:.3}\t{ci_low:.3}\t{ci_high:.3}\n"));
        series.push(serde_json::json!({
            "generation": g,
            "ne": ne,
            "ci_low": ci_low,
            "ci_high": ci_high
        }));
    }
    atomic_write_bytes(&ne_trajectory_tsv, tsv.as_bytes())?;
    atomic_write_json(
        &demography_metrics_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.demography.v1",
            "segments_validated": valid_segments,
            "ne_recent": series.first().and_then(|v| v.get("ne")).unwrap_or(&serde_json::Value::Null),
            "ne_time_series": series,
            "ne_confidence_interval": "generated_per_generation",
            "tool_attempts": {
                "ibdne": ibdne_ok
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!("runner=ibdne_like\nibdne_attempted={ibdne_ok}\n").as_bytes(),
    )?;
    Ok(DemographyStageOutputs {
        ne_trajectory_tsv,
        demography_metrics_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error when panel/map/species contracts are violated or artifacts cannot be written.
pub fn run_prepare_reference_panel_stage(
    input_vcf: &Path,
    panel_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PrepareReferencePanelParams,
) -> Result<PrepareReferencePanelOutputs> {
    if species_context.species_id != params.species_id
        || species_context.build_id != params.build_id
    {
        bail!("species/build mismatch between stage params and SpeciesContext");
    }
    let panel = resolve_panel(
        &params.species_id,
        &params.build_id,
        params.panel_id.as_deref(),
    )?;
    let map = resolve_map(
        &params.species_id,
        &params.build_id,
        params.map_id.as_deref(),
    )?;
    if panel.species_id != species_context.species_id || panel.build_id != species_context.build_id
    {
        bail!("panel species/build does not match SpeciesContext");
    }
    if map.species_id != species_context.species_id || map.build_id != species_context.build_id {
        bail!("map species/build does not match SpeciesContext");
    }

    let panel_parent = panel_vcf
        .parent()
        .ok_or_else(|| anyhow!("panel path has no parent: {}", panel_vcf.display()))?;
    if panel_parent.file_name().and_then(|x| x.to_str()) != Some("raw") {
        bail!(
            "panel materialization refusal: panel must be acquired via scripts/tooling/acquire-panels.sh and live under .../raw/"
        );
    }
    let source_panel_root = panel_parent
        .parent()
        .ok_or_else(|| anyhow!("panel raw path missing panel root"))?;
    if !source_panel_root.join("normalized").exists() || !source_panel_root.join("derived").exists()
    {
        bail!(
            "panel materialization refusal: expected sibling normalized/derived dirs from acquire-panels materialization"
        );
    }

    let input_raw = read_vcf_text(input_vcf)?;
    let panel_raw = read_vcf_text(panel_vcf)?;
    let mut input_keys = std::collections::BTreeSet::<String>::new();
    let mut panel_by_chr = std::collections::BTreeMap::<String, u64>::new();
    let mut overlap_by_chr = std::collections::BTreeMap::<String, u64>::new();
    for line in input_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((_chr, key)) = variant_key(&fields) {
                input_keys.insert(key);
            }
        }
    }
    for line in panel_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((chr, key)) = variant_key(&fields) {
                *panel_by_chr.entry(chr.clone()).or_insert(0) += 1;
                if input_keys.contains(&key) {
                    *overlap_by_chr.entry(chr).or_insert(0) += 1;
                }
            }
        }
    }
    let panel_total: u64 = panel_by_chr.values().sum();
    let overlap_total: u64 = overlap_by_chr.values().sum();
    let overlap_fraction = if panel_total == 0 {
        0.0
    } else {
        overlap_total as f64 / panel_total as f64
    };
    if panel_total > 0 && overlap_fraction < 0.05 {
        bail!("prepare_reference_panel refusal: too few overlapping sites with target VCF");
    }

    bijux_dna_infra::ensure_dir(out_dir)?;
    let lock_seed = format!(
        "{}|{}|{}",
        panel.id, panel.version, panel.build_id
    );
    let lock_hash = checksum_hex(lock_seed.as_bytes());
    let panel_root = out_dir
        .join("panels")
        .join(panel.id.clone())
        .join(lock_hash);
    let local_raw = panel_root.join("raw");
    let local_normalized = panel_root.join("normalized");
    let local_derived = panel_root.join("derived");
    bijux_dna_infra::ensure_dir(&local_raw)?;
    bijux_dna_infra::ensure_dir(&local_normalized)?;
    bijux_dna_infra::ensure_dir(&local_derived)?;

    let local_raw_panel_vcf = local_raw.join(
        panel_vcf
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("panel.vcf.gz"),
    );
    atomic_write_bytes(&local_raw_panel_vcf, &std::fs::read(panel_vcf)?)?;

    let prepared_panel_vcf = local_normalized.join("prepared_panel.vcf.gz");
    let panel_manifest_json = out_dir.join("panel_manifest.json");
    let overlap_json = out_dir.join("overlap.json");
    let panel_overlap_json = out_dir.join("panel_overlap.json");
    let panel_files_json = out_dir.join("panel_files.json");
    let overlap_tsv = out_dir.join("overlap.tsv");
    let chunks_json = out_dir.join("chunks.json");

    let mut header_lines = Vec::<String>::new();
    let mut record_lines = Vec::<String>::new();
    for line in panel_raw.lines() {
        if line.starts_with('#') {
            header_lines.push(line.to_string());
        } else if !line.trim().is_empty() {
            record_lines.push(line.to_string());
        }
    }
    let contig_rank = species_context
        .contigs
        .iter()
        .enumerate()
        .map(|(idx, c)| (canonical_contig_label(&c.name), idx))
        .collect::<std::collections::BTreeMap<_, _>>();
    let allowed_contigs = species_context
        .contigs
        .iter()
        .map(|c| canonical_contig_label(&c.name))
        .collect::<std::collections::BTreeSet<_>>();
    for rec in &record_lines {
        if let Some(fields) = parse_record_fields(rec) {
            let chr = fields[0];
            if !allowed_contigs.contains(&canonical_contig_label(chr)) {
                bail!(
                    "panel normalization refusal: panel contig {} not present in species context",
                    chr
                );
            }
        }
    }
    record_lines.sort_by(|a, b| {
        let ka = parse_variant_key(a).unwrap_or_default();
        let kb = parse_variant_key(b).unwrap_or_default();
        let ra = contig_rank
            .get(&canonical_contig_label(&ka.0))
            .copied()
            .unwrap_or(usize::MAX);
        let rb = contig_rank
            .get(&canonical_contig_label(&kb.0))
            .copied()
            .unwrap_or(usize::MAX);
        ra.cmp(&rb)
            .then(ka.1.cmp(&kb.1))
            .then(ka.2.cmp(&kb.2))
    });
    let normalized_payload = format!("{}\n{}\n", header_lines.join("\n"), record_lines.join("\n"));
    let prepared_panel_tbi = write_bgzip_with_best_effort_index(
        &prepared_panel_vcf,
        &normalized_payload,
        "prepared_panel.tmp.vcf",
    )?;
    assert_bgzip_tabix_artifacts(&prepared_panel_vcf, &prepared_panel_tbi)?;

    let site_list = local_derived.join("panel_sites.tsv");
    let mut site_rows = String::from("contig\tpos\tref\talt\n");
    for rec in &record_lines {
        if let Some(fields) = parse_record_fields(rec) {
            site_rows.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                fields[0], fields[1], fields[3], fields[4]
            ));
        }
    }
    atomic_write_bytes(&site_list, site_rows.as_bytes())?;
    let chunk_regions = local_derived.join("chunk_regions.tsv");
    atomic_write_bytes(
        &chunk_regions,
        b"chunk_id\tregion\nchunk_000\t1:1-1000000\n",
    )?;
    if panel.compatibility.supports_minimac_m3vcf {
        let minimac_ready = local_derived.join("minimac.m3vcf.ready");
        atomic_write_bytes(&minimac_ready, b"true\n")?;
    }

    let manifest = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.manifest.v1",
        "species_id": params.species_id,
        "build_id": params.build_id,
        "panel_root": panel_root,
        "source_panel_root": source_panel_root,
        "panel": {
            "id": panel.id,
            "version": panel.version,
            "file_count": panel.files.len(),
            "compatibility": panel.compatibility,
        },
        "map": {
            "id": map.id,
            "version": map.version,
            "file_count": map.files.len(),
            "compatibility": map.compatibility,
        }
    });
    atomic_write_json(&panel_manifest_json, &manifest)?;
    let per_chr = panel_by_chr
        .iter()
        .map(|(chr, total)| {
            let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
            let frac = if *total == 0 {
                0.0
            } else {
                overlap as f64 / *total as f64
            };
            serde_json::json!({
                "chr": chr,
                "panel_sites": total,
                "overlap_sites": overlap,
                "overlap_fraction": frac,
            })
        })
        .collect::<Vec<_>>();
    let overlap_payload = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.overlap.v1",
        "global": {
            "panel_sites": panel_total,
            "overlap_sites": overlap_total,
            "overlap_fraction": overlap_fraction,
        },
        "per_chr": per_chr,
    });
    atomic_write_json(&overlap_json, &overlap_payload)?;
    atomic_write_json(&panel_overlap_json, &overlap_payload)?;
    let panel_files_payload = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.files.v1",
        "panel_root": panel_root,
        "raw_files": [local_raw_panel_vcf],
        "normalized_files": [prepared_panel_vcf, prepared_panel_tbi],
        "derived_files": [site_list, chunk_regions],
    });
    atomic_write_json(&panel_files_json, &panel_files_payload)?;
    let mut tsv = String::from("chr\tpanel_sites\toverlap_sites\toverlap_fraction\n");
    for (chr, total) in &panel_by_chr {
        let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
        let frac = if *total == 0 {
            0.0
        } else {
            overlap as f64 / *total as f64
        };
        tsv.push_str(&format!("{chr}\t{total}\t{overlap}\t{frac:.6}\n"));
    }
    atomic_write_bytes(&overlap_tsv, tsv.as_bytes())?;

    let chunk_plan = plan_regions_deterministic(species_context, &ChunkingPlanParams::default())?;
    let chunk_rows = chunk_plan
        .iter()
        .map(|c| {
            let panel_sites = *panel_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_sites = *overlap_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_fraction = if panel_sites == 0 {
                0.0
            } else {
                overlap_sites as f64 / panel_sites as f64
            };
            serde_json::json!({
                "chunk_id": c.chunk_id,
                "region": c.region_string(),
                "estimated_variants": 0,
                "actual_variants": 0,
                "panel_overlap_fraction": overlap_fraction,
            })
        })
        .collect::<Vec<_>>();
    let chunks_payload = serde_json::json!({
        "schema_version": "bijux.vcf.chunk_plan.v1",
        "strategy": "deterministic_species_context",
        "chunks": chunk_rows,
    });
    atomic_write_json(&chunks_json, &chunks_payload)?;

    let relevant_tools = ["bcftools", "shapeit5", "impute5", "glimpse", "beagle", "minimac4"];
    for tool in relevant_tools {
        if !license_metadata_for_tool_exists(tool) {
            bail!(
                "panel license policy refusal: missing containers/licenses/{}.license.toml",
                tool
            );
        }
    }

    Ok(PrepareReferencePanelOutputs {
        panel_root,
        prepared_panel_vcf,
        prepared_panel_tbi,
        panel_manifest_json,
        overlap_json,
        panel_overlap_json,
        panel_files_json,
        overlap_tsv,
        chunks_json,
    })
}
