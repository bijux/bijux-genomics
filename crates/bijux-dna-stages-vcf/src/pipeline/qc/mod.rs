use super::*;

#[derive(Debug, Clone)]
pub struct QcStageParams {
    pub sample_name: String,
    pub is_ancient_dna: bool,
    pub allow_hwe_for_ancient: bool,
    pub production_profile: bool,
    pub pre_filter_vcf: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QcStageOutputs {
    pub qc_summary_json: PathBuf,
    pub qc_tables_tsv: PathBuf,
    pub imputation_qc_tsv: PathBuf,
    pub warnings_json: PathBuf,
    pub qc_histograms_json: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct QcSampleMissingnessRow {
    sample_id: String,
    total_genotype_count: u64,
    missing_genotype_count: u64,
    missingness: f64,
}

#[derive(Debug, Clone, Serialize)]
struct QcVariantMissingnessRow {
    variant_id: String,
    contig: String,
    position: u64,
    reference: String,
    alternate: String,
    total_sample_count: u64,
    missing_sample_count: u64,
    missingness: f64,
}

fn maf_bin_label(maf: f64) -> &'static str {
    if maf < 0.01 {
        "0-0.01"
    } else if maf < 0.05 {
        "0.01-0.05"
    } else if maf < 0.1 {
        "0.05-0.1"
    } else if maf < 0.2 {
        "0.1-0.2"
    } else {
        "0.2-0.5"
    }
}

fn parse_gt_counts(format_field: &str, sample_fields: &[&str]) -> Option<(u64, u64, u64, u64)> {
    let keys = format_field.split(':').collect::<Vec<_>>();
    let gt_idx = keys.iter().position(|k| *k == "GT")?;
    let mut hom_ref = 0_u64;
    let mut het = 0_u64;
    let mut hom_alt = 0_u64;
    let mut total = 0_u64;
    for sample in sample_fields {
        let vals = sample.split(':').collect::<Vec<_>>();
        let Some(gt) = vals.get(gt_idx) else {
            continue;
        };
        if gt.contains('.') {
            continue;
        }
        total += 1;
        match gt.replace('|', "/").as_str() {
            "0/0" => hom_ref += 1,
            "0/1" | "1/0" => het += 1,
            "1/1" => hom_alt += 1,
            _ => {}
        }
    }
    Some((hom_ref, het, hom_alt, total))
}

fn is_transition(reference: &str, alt: &str) -> bool {
    matches!(
        (reference.to_ascii_uppercase().as_str(), alt.to_ascii_uppercase().as_str()),
        ("A", "G") | ("G", "A") | ("C", "T") | ("T", "C")
    )
}

fn is_transversion(reference: &str, alt: &str) -> bool {
    !is_transition(reference, alt)
}

/// # Errors
/// Returns an error if QC metrics cannot be computed or fail production thresholds.
pub fn run_qc_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &QcStageParams,
) -> Result<QcStageOutputs> {
    if params.is_ancient_dna && !params.allow_hwe_for_ancient {
        // HWE and other modern-only metrics are intentionally skipped by default for aDNA.
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut depth = std::collections::BTreeMap::<String, u64>::new();
    let mut info_values = Vec::<f64>::new();
    let mut rsq_values = Vec::<f64>::new();
    let mut af_values = Vec::<f64>::new();
    let mut maf_bins = std::collections::BTreeMap::<String, u64>::new();
    let mut missing = 0_u64;
    let mut called = 0_u64;
    let mut variants = 0_u64;
    let mut site_missingness = Vec::<f64>::new();
    let mut hwe_p_values = Vec::<f64>::new();
    let mut ti_count = 0_u64;
    let mut tv_count = 0_u64;
    let mut het_total = 0_u64;
    let mut hom_alt_total = 0_u64;
    let mut per_sample = std::collections::BTreeMap::<String, (u64, u64)>::new();
    let mut filter_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut rsq_by_maf_bin_sum = std::collections::BTreeMap::<String, f64>::new();
    let mut rsq_by_maf_bin_count = std::collections::BTreeMap::<String, u64>::new();
    let mut post_variant_keys = std::collections::BTreeSet::<String>::new();
    let mut sample_ids = Vec::<String>::new();
    let mut variant_missingness_rows = Vec::<QcVariantMissingnessRow>::new();
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            sample_ids =
                line.split('\t').skip(9).map(std::string::ToString::to_string).collect::<Vec<_>>();
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        variants += 1;
        let filter_label = if fields[6].trim().is_empty() || fields[6] == "." {
            "PASS".to_string()
        } else {
            fields[6].to_string()
        };
        *filter_counts.entry(filter_label).or_insert(0) += 1;
        post_variant_keys
            .insert(format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]));
        if let (Some(reference), Some(alt)) = (fields.get(3), fields.get(4)) {
            if is_transition(reference, alt) {
                ti_count += 1;
            } else if is_transversion(reference, alt) {
                tv_count += 1;
            }
        }
        if let Some(dp) = parse_depth_from_info(fields[7]) {
            let bucket = if dp < 10 {
                "0-9"
            } else if dp < 20 {
                "10-19"
            } else if dp < 30 {
                "20-29"
            } else {
                "30+"
            };
            *depth.entry(bucket.to_string()).or_insert(0) += 1;
        }
        if let Some(v) = parse_info_value_f64(fields[7], "INFO") {
            info_values.push(v);
        }
        if let Some(v) = parse_info_value_f64(fields[7], "R2") {
            rsq_values.push(v);
        }
        if let Some(v) = parse_af_from_info(fields[7]) {
            af_values.push(v);
            let maf_bin = maf_bin_label(v).to_string();
            *maf_bins.entry(maf_bin.clone()).or_insert(0) += 1;
            if let Some(rsq) = parse_info_value_f64(fields[7], "R2") {
                *rsq_by_maf_bin_sum.entry(maf_bin.clone()).or_insert(0.0) += rsq;
                *rsq_by_maf_bin_count.entry(maf_bin).or_insert(0) += 1;
            }
        }
        if fields.len() > 9 {
            let keys = fields[8].split(':').collect::<Vec<_>>();
            if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
                let mut site_missing = 0_u64;
                let mut site_total = 0_u64;
                for (sample_index, sample) in fields[9..].iter().enumerate() {
                    let vals = sample.split(':').collect::<Vec<_>>();
                    if let Some(gt) = vals.get(gt_idx) {
                        site_total += 1;
                        let sample_id = sample_ids
                            .get(sample_index)
                            .cloned()
                            .unwrap_or_else(|| format!("sample{}", sample_index + 1));
                        let entry = per_sample.entry(sample_id).or_insert((0, 0));
                        entry.0 += 1;
                        if gt.contains('.') {
                            missing += 1;
                            site_missing += 1;
                            entry.1 += 1;
                        } else {
                            called += 1;
                            match gt.replace('|', "/").as_str() {
                                "0/1" | "1/0" => het_total += 1,
                                "1/1" => hom_alt_total += 1,
                                _ => {}
                            }
                        }
                    }
                }
                if site_total > 0 {
                    let missingness = site_missing as f64 / site_total as f64;
                    site_missingness.push(missingness);
                    let variant_id = if !fields[2].trim().is_empty() && fields[2] != "." {
                        fields[2].to_string()
                    } else {
                        format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4])
                    };
                    variant_missingness_rows.push(QcVariantMissingnessRow {
                        variant_id,
                        contig: fields[0].to_string(),
                        position: fields[1].parse::<u64>().unwrap_or(0),
                        reference: fields[3].to_string(),
                        alternate: fields[4].to_string(),
                        total_sample_count: site_total,
                        missing_sample_count: site_missing,
                        missingness,
                    });
                }
            }
            if !params.is_ancient_dna || params.allow_hwe_for_ancient {
                if let Some((hom_ref, het, hom_alt, total)) =
                    parse_gt_counts(fields[8], &fields[9..])
                {
                    if total > 0 {
                        let n = total as f64;
                        let p = (2.0 * hom_ref as f64 + het as f64) / (2.0 * n);
                        let q = 1.0 - p;
                        let e_hom_ref = n * p * p;
                        let e_het = 2.0 * n * p * q;
                        let e_hom_alt = n * q * q;
                        if e_hom_ref > 0.0 && e_het > 0.0 && e_hom_alt > 0.0 {
                            let chi2 = (hom_ref as f64 - e_hom_ref).powi(2) / e_hom_ref
                                + (het as f64 - e_het).powi(2) / e_het
                                + (hom_alt as f64 - e_hom_alt).powi(2) / e_hom_alt;
                            let p_approx = (-0.5 * chi2).exp();
                            hwe_p_values.push(p_approx);
                        }
                    }
                }
            }
        }
    }
    let missingness_post =
        if called + missing == 0 { 0.0 } else { missing as f64 / (called + missing) as f64 };
    let missingness_pre = if let Some(pre) = &params.pre_filter_vcf {
        let pre_raw = read_vcf_text(pre)?;
        let mut pre_missing = 0_u64;
        let mut pre_called = 0_u64;
        for line in pre_raw.lines() {
            let Some(fields) = parse_record_fields(line) else {
                continue;
            };
            if fields.len() <= 9 {
                continue;
            }
            let keys = fields[8].split(':').collect::<Vec<_>>();
            if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
                for sample in &fields[9..] {
                    let vals = sample.split(':').collect::<Vec<_>>();
                    if let Some(gt) = vals.get(gt_idx) {
                        if gt.contains('.') {
                            pre_missing += 1;
                        } else {
                            pre_called += 1;
                        }
                    }
                }
            }
        }
        if pre_missing + pre_called == 0 {
            missingness_post
        } else {
            pre_missing as f64 / (pre_missing + pre_called) as f64
        }
    } else {
        missingness_post
    };
    let info_mean = if info_values.is_empty() {
        0.0
    } else {
        info_values.iter().sum::<f64>() / info_values.len() as f64
    };
    let rsq_mean = if rsq_values.is_empty() {
        0.0
    } else {
        rsq_values.iter().sum::<f64>() / rsq_values.len() as f64
    };
    let af_mean = if af_values.is_empty() {
        0.0
    } else {
        af_values.iter().sum::<f64>() / af_values.len() as f64
    };
    let site_missingness_mean = if site_missingness.is_empty() {
        0.0
    } else {
        site_missingness.iter().sum::<f64>() / site_missingness.len() as f64
    };
    let hwe_p_mean = if hwe_p_values.is_empty() {
        None
    } else {
        Some(hwe_p_values.iter().sum::<f64>() / hwe_p_values.len() as f64)
    };
    let ti_tv = if tv_count == 0 { None } else { Some(ti_count as f64 / tv_count as f64) };
    let het_hom_ratio =
        if hom_alt_total == 0 { None } else { Some(het_total as f64 / hom_alt_total as f64) };
    let thresholds = load_imputation_qc_thresholds();
    let sample_missingness_exclusion_threshold =
        *thresholds.get("vcf_qc_sample_missingness_exclude").unwrap_or(&0.50);
    let variant_missingness_exclusion_threshold =
        *thresholds.get("vcf_qc_variant_missingness_exclude").unwrap_or(&0.50);
    if params.production_profile {
        if missingness_post > *thresholds.get("vcf_qc_missingness_post_fail").unwrap_or(&0.15) {
            bail!("vcf.qc production gate failed: missingness_post above fail threshold");
        }
        if !info_values.is_empty()
            && info_mean < *thresholds.get("vcf_qc_info_fail").unwrap_or(&0.60)
        {
            bail!("vcf.qc production gate failed: imputation INFO mean below fail threshold");
        }
    }
    let qc_tables_tsv = out_dir.join("qc_tables.tsv");
    let imputation_qc_tsv = out_dir.join("imputation_qc.tsv");
    let mut table = String::from("metric\tvalue\n");
    table.push_str(&format!("sample_name\t{}\n", params.sample_name));
    table.push_str(&format!("variants\t{variants}\n"));
    table.push_str(&format!("missingness_pre\t{missingness_pre:.6}\n"));
    table.push_str(&format!("missingness_post\t{missingness_post:.6}\n"));
    table.push_str(&format!("allele_freq_mean\t{af_mean:.6}\n"));
    table.push_str(&format!("site_missingness_mean\t{site_missingness_mean:.6}\n"));
    table.push_str(&format!("imputation_info_mean\t{info_mean:.6}\n"));
    table.push_str(&format!("rsq_mean\t{rsq_mean:.6}\n"));
    for (bin, count) in &maf_bins {
        table.push_str(&format!("maf_bin_count_{bin}\t{count}\n"));
        let rsq_mean_bin = if *rsq_by_maf_bin_count.get(bin).unwrap_or(&0) == 0 {
            0.0
        } else {
            rsq_by_maf_bin_sum.get(bin).copied().unwrap_or(0.0)
                / *rsq_by_maf_bin_count.get(bin).unwrap_or(&1) as f64
        };
        table.push_str(&format!("rsq_mean_{bin}\t{rsq_mean_bin:.6}\n"));
    }
    for (filter, count) in &filter_counts {
        table.push_str(&format!("filter_count_{filter}\t{count}\n"));
    }
    if let Some(hwe) = hwe_p_mean {
        table.push_str(&format!("hwe_pvalue_mean\t{hwe:.6}\n"));
    }
    table.push_str(&format!(
        "hwe_status\t{}\n",
        if params.is_ancient_dna && !params.allow_hwe_for_ancient {
            "skipped_ancient_default"
        } else {
            "computed_modern"
        }
    ));
    atomic_write_bytes(&qc_tables_tsv, table.as_bytes())?;
    atomic_write_bytes(&imputation_qc_tsv, table.as_bytes())?;
    let warnings_json = out_dir.join("warnings.json");
    let overlap_diagnostics = if let Some(pre) = &params.pre_filter_vcf {
        let pre_raw = read_vcf_text(pre)?;
        let mut pre_variant_keys = std::collections::BTreeSet::<String>::new();
        for line in pre_raw.lines() {
            let Some(fields) = parse_record_fields(line) else {
                continue;
            };
            pre_variant_keys
                .insert(format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]));
        }
        let shared = pre_variant_keys.intersection(&post_variant_keys).count() as u64;
        let pre_total = pre_variant_keys.len() as u64;
        let post_total = post_variant_keys.len() as u64;
        let post_overlap_fraction =
            if post_total == 0 { 0.0 } else { shared as f64 / post_total as f64 };
        serde_json::json!({
            "pre_total": pre_total,
            "post_total": post_total,
            "shared_variants": shared,
            "post_overlap_fraction": post_overlap_fraction
        })
    } else {
        serde_json::json!({
            "pre_total": serde_json::Value::Null,
            "post_total": post_variant_keys.len(),
            "shared_variants": serde_json::Value::Null,
            "post_overlap_fraction": serde_json::Value::Null
        })
    };
    let mut sample_missingness = per_sample
        .iter()
        .map(|(sample_id, (total, miss))| {
            let missingness = if *total == 0 { 0.0 } else { *miss as f64 / *total as f64 };
            QcSampleMissingnessRow {
                sample_id: sample_id.clone(),
                total_genotype_count: *total,
                missing_genotype_count: *miss,
                missingness,
            }
        })
        .collect::<Vec<_>>();
    sample_missingness.sort_by(|a, b| a.sample_id.cmp(&b.sample_id));
    let mean_sample_missing = if sample_missingness.is_empty() {
        0.0
    } else {
        sample_missingness.iter().map(|row| row.missingness).sum::<f64>()
            / sample_missingness.len() as f64
    };
    let var_sample_missing = if sample_missingness.len() < 2 {
        0.0
    } else {
        sample_missingness
            .iter()
            .map(|row| (row.missingness - mean_sample_missing).powi(2))
            .sum::<f64>()
            / sample_missingness.len() as f64
    };
    let std_sample_missing = var_sample_missing.sqrt();
    let outlier_cutoff = mean_sample_missing + (3.0 * std_sample_missing);
    let excluded_samples = sample_missingness
        .iter()
        .filter(|row| row.missingness > sample_missingness_exclusion_threshold)
        .cloned()
        .collect::<Vec<_>>();
    let excluded_variants = variant_missingness_rows
        .iter()
        .filter(|row| row.missingness > variant_missingness_exclusion_threshold)
        .cloned()
        .collect::<Vec<_>>();
    let per_sample_outliers = sample_missingness
        .iter()
        .filter(|row| row.missingness > outlier_cutoff && std_sample_missing > 0.0)
        .map(|row| serde_json::json!({"sample": row.sample_id, "missingness": row.missingness}))
        .collect::<Vec<_>>();
    atomic_write_json(
        &warnings_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.qc_warnings.v1",
            "warnings": if overlap_diagnostics.get("post_overlap_fraction").and_then(|x| x.as_f64()).is_some_and(|x| x < 0.95) {
                vec!["low_pre_post_variant_overlap".to_string()]
            } else {
                Vec::<String>::new()
            },
            "per_sample_outliers": per_sample_outliers,
            "overlap_diagnostics": overlap_diagnostics.clone(),
            "excluded_samples": excluded_samples.clone(),
            "excluded_variants": excluded_variants.clone(),
            "sample_missingness_exclusion_threshold": sample_missingness_exclusion_threshold,
            "variant_missingness_exclusion_threshold": variant_missingness_exclusion_threshold,
        }),
    )?;
    let rsq_by_maf_bin = maf_bins
        .keys()
        .map(|bin| {
            let count = *rsq_by_maf_bin_count.get(bin).unwrap_or(&0);
            let mean = if count == 0 {
                0.0
            } else {
                rsq_by_maf_bin_sum.get(bin).copied().unwrap_or(0.0) / count as f64
            };
            (
                bin.clone(),
                serde_json::json!({
                    "count": count,
                    "rsq_mean": mean
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let qc_histograms_json = out_dir.join("qc_histograms.json");
    atomic_write_json(
        &qc_histograms_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.qc_histograms.v1",
            "depth_distribution": depth.clone(),
            "info_distribution": info_values.clone(),
            "rsq_distribution": rsq_values.clone(),
            "maf_bins": maf_bins.clone(),
            "rsq_by_maf_bin": rsq_by_maf_bin.clone(),
            "site_missingness_distribution": site_missingness.clone(),
            "filter_counts": filter_counts.clone(),
        }),
    )?;
    let qc_summary_json = out_dir.join("qc_summary.json");
    atomic_write_json(
        &qc_summary_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.qc.v1",
            "missingness_pre": missingness_pre,
            "missingness_post": missingness_post,
            "imputation_info_mean": info_mean,
            "rsq_mean": rsq_mean,
            "allele_frequency_shift_abs_mean": af_mean,
            "maf_bins": maf_bins,
            "rsq_by_maf_bin": rsq_by_maf_bin,
            "site_missingness_mean": site_missingness_mean,
            "depth_distribution": depth,
            "filter_counts": filter_counts,
            "overlap_diagnostics": overlap_diagnostics.clone(),
            "hwe_pvalue_mean": hwe_p_mean,
            "ti_tv": ti_tv,
            "het_hom_ratio": het_hom_ratio,
            "hwe_status": if params.is_ancient_dna && !params.allow_hwe_for_ancient { "skipped_ancient_default" } else { "computed_modern" },
            "sample_missingness": sample_missingness.clone(),
            "variant_missingness": variant_missingness_rows.clone(),
            "maf_summary": {
                "allele_frequency_mean": af_mean,
                "maf_bin_counts": maf_bins.clone(),
                "observed_variant_count": variants,
            },
            "heterozygosity": {
                "heterozygous_call_count": het_total,
                "homozygous_alt_call_count": hom_alt_total,
                "het_hom_ratio": het_hom_ratio,
            },
            "excluded_samples": excluded_samples.clone(),
            "excluded_variants": excluded_variants.clone(),
            "sample_missingness_exclusion_threshold": sample_missingness_exclusion_threshold,
            "variant_missingness_exclusion_threshold": variant_missingness_exclusion_threshold,
        }),
    )?;
    Ok(QcStageOutputs {
        qc_summary_json,
        qc_tables_tsv,
        imputation_qc_tsv,
        warnings_json,
        qc_histograms_json,
    })
}

/// # Errors
/// Returns an error if stats cannot be computed or written.
pub fn run_stats_stage(
    input_vcf: &Path,
    output_stats: &Path,
    params: &VcfStatsParams,
) -> Result<VcfStatsMetricsV1> {
    let out_dir = output_stats
        .parent()
        .ok_or_else(|| anyhow!("vcf.stats output path has no parent directory"))?;
    let out = run_stats_stage_real(input_vcf, out_dir, params)?;
    std::fs::copy(out.stats_json, output_stats)?;
    Ok(out.metrics)
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsStageOutputs {
    pub bcftools_stats_txt: PathBuf,
    pub stats_json: PathBuf,
    pub metrics: VcfStatsMetricsV1,
}

/// # Errors
/// Returns an error if stats artifacts cannot be computed/written.
pub fn run_stats_stage_real(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfStatsParams,
) -> Result<StatsStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let source_vcfgz = if input_vcf
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x == "gz" || x == "bcf")
    {
        input_vcf.to_path_buf()
    } else {
        let normalized_input = out_dir.join("stats_input.normalized.vcf.gz");
        let plain_input = out_dir.join("stats_input.normalized.vcf");
        std::fs::copy(input_vcf, &plain_input)?;
        let _ = crate::vcf_io::vcf_index_bgzip_tabix(&plain_input, &normalized_input)?;
        normalized_input
    };
    let bcftools_stats_txt = out_dir.join("bcftools_stats.txt");
    let mut metrics =
        if let Ok(real) = crate::vcf_io::vcf_stats_basic(&source_vcfgz, &bcftools_stats_txt) {
            real
        } else {
            let call = parse_vcf_call_summary(input_vcf, &params.sample_name)?;
            let filter = parse_vcf_filter_breakdown(input_vcf, &params.sample_name)?;
            VcfStatsMetricsV1 {
                schema_version: "bijux.vcf.stats.v1".to_string(),
                sample_name: params.sample_name.clone(),
                variants_total: call.variants_called,
                sample_count: 0,
                snps: call.snps,
                indels: call.indels,
                ti_tv: None,
                missingness_post: None,
                heterozygosity_ratio: None,
                annotation_coverage: None,
                filter_breakdown: filter.filter_breakdown.clone(),
                depth_distribution: std::collections::BTreeMap::new(),
                call_summary: call,
                filter_summary: filter,
            }
        };
    metrics.sample_name = params.sample_name.clone();
    enrich_stats_metrics_from_vcf(input_vcf, &mut metrics)?;
    let stats_json = out_dir.join("stats.json");
    atomic_write_json(&stats_json, &serde_json::to_value(&metrics)?)?;
    Ok(StatsStageOutputs { bcftools_stats_txt, stats_json, metrics })
}

fn enrich_stats_metrics_from_vcf(input_vcf: &Path, metrics: &mut VcfStatsMetricsV1) -> Result<()> {
    let raw = read_vcf_text(input_vcf)?;
    let mut sample_count = 0_u64;
    let mut annotated_records = 0_u64;
    let mut called = 0_u64;
    let mut missing = 0_u64;
    let mut het_total = 0_u64;
    let mut hom_alt_total = 0_u64;
    let mut ti_count = 0_u64;
    let mut tv_count = 0_u64;

    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            sample_count = line.split('\t').skip(9).count() as u64;
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        if fields[3].len() == 1 && fields[4].len() == 1 {
            if is_transition(fields[3], fields[4]) {
                ti_count += 1;
            } else {
                tv_count += 1;
            }
        }
        if fields[7] != "." && !fields[7].trim().is_empty() {
            annotated_records += 1;
        }
        if fields.len() <= 9 {
            continue;
        }
        let keys = fields[8].split(':').collect::<Vec<_>>();
        if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
            for sample in &fields[9..] {
                let vals = sample.split(':').collect::<Vec<_>>();
                if let Some(gt) = vals.get(gt_idx) {
                    if gt.contains('.') {
                        missing += 1;
                    } else {
                        called += 1;
                        match gt.replace('|', "/").as_str() {
                            "0/1" | "1/0" => het_total += 1,
                            "1/1" => hom_alt_total += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    metrics.sample_count = sample_count;
    if called + missing > 0 {
        metrics.missingness_post = Some(missing as f64 / (called + missing) as f64);
    }
    if hom_alt_total > 0 {
        metrics.heterozygosity_ratio = Some(het_total as f64 / hom_alt_total as f64);
    }
    if metrics.variants_total > 0 {
        metrics.annotation_coverage =
            Some(annotated_records as f64 / metrics.variants_total as f64);
    }
    if metrics.ti_tv.is_none() && tv_count > 0 {
        metrics.ti_tv = Some(ti_count as f64 / tv_count as f64);
    }
    Ok(())
}

// Errors are surfaced from helpers and include deserialization/index checks.
mod stage_params;

pub use stage_params::*;
