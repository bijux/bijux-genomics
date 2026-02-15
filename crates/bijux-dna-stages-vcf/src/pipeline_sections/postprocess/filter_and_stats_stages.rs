pub fn run_filter_stage(
    input_vcf: &Path,
    output_vcf: &Path,
    params: &VcfFilterParams,
) -> Result<()> {
    let out_dir = output_vcf
        .parent()
        .ok_or_else(|| anyhow!("vcf.filter output path has no parent directory"))?;
    let _ = run_filter_stage_real(input_vcf, out_dir, params)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct FilterStageOutputs {
    pub filtered_vcf: PathBuf,
    pub filtered_tbi: PathBuf,
    pub filter_breakdown_json: PathBuf,
    pub filter_breakdown_tsv: PathBuf,
}

fn parse_af_from_info(info: &str) -> Option<f64> {
    parse_info_value_f64(info, "AF").or_else(|| parse_info_value_f64(info, "MAF"))
}

fn genotype_missing_fraction(format_field: &str, sample_fields: &[&str]) -> Option<f64> {
    let keys = format_field.split(':').collect::<Vec<_>>();
    let gt_idx = keys.iter().position(|k| *k == "GT")?;
    if sample_fields.is_empty() {
        return Some(0.0);
    }
    let mut missing = 0_u64;
    let mut total = 0_u64;
    for sample in sample_fields {
        let vals = sample.split(':').collect::<Vec<_>>();
        if let Some(gt) = vals.get(gt_idx) {
            total += 1;
            if gt.contains('.') {
                missing += 1;
            }
        }
    }
    Some(if total == 0 { 0.0 } else { missing as f64 / total as f64 })
}

/// # Errors
/// Returns an error if filter stage outputs cannot be materialized.
pub fn run_filter_stage_real(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfFilterParams,
) -> Result<FilterStageOutputs> {
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut out = String::new();
    let mut kept = 0u64;
    let mut tag_counts = std::collections::BTreeMap::<String, u64>::new();
    let maf_min = 0.01_f64;
    let sample_missingness_max = 0.20_f64;
    let expression = format!(
        "QUAL>={:.3} && F_MISSING<={:.3} && (AF>={:.3} || AF missing)",
        params.min_qual, sample_missingness_max, maf_min
    );
    let mut total_records = 0_u64;
    for line in raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            total_records += 1;
            let qual = fields[5].parse::<f64>().unwrap_or(0.0);
            let af = parse_af_from_info(fields[7]);
            let f_missing = if fields.len() > 9 {
                genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0)
            } else {
                0.0
            };
            let mut reasons = Vec::<&str>::new();
            if qual < params.min_qual {
                reasons.push("LOWQUAL");
            }
            if f_missing > sample_missingness_max {
                reasons.push("HIGH_MISSING");
            }
            if let Some(x) = af {
                if x < maf_min {
                    reasons.push("LOW_MAF");
                }
            }
            if reasons.is_empty() {
                *tag_counts.entry("PASS".to_string()).or_insert(0) += 1;
            } else {
                for reason in &reasons {
                    *tag_counts.entry((*reason).to_string()).or_insert(0) += 1;
                }
            }
            if params.require_pass && !reasons.is_empty() {
                continue;
            }
            let mut row = fields.iter().copied().map(str::to_string).collect::<Vec<_>>();
            row[6] = if reasons.is_empty() {
                "PASS".to_string()
            } else {
                reasons.join(";")
            };
            if params.normalize {
                let (r, a) = normalize_alleles(&row[3], &row[4]);
                row[3] = r;
                row[4] = a;
            }
            kept += 1;
            out.push_str(&row.join("\t"));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if params.production_profile && kept == 0 {
        return Err(anyhow!(
            "vcf.filter removed all variants in production_profile mode"
        ));
    }
    if params.production_profile && total_records > 0 {
        let retention = kept as f64 / total_records as f64;
        let fail = *load_imputation_qc_thresholds()
            .get("vcf_filter_retention_fail")
            .unwrap_or(&0.20);
        if retention < fail {
            bail!("vcf.filter production gate failed: retention below fail threshold");
        }
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let filtered_vcf = out_dir.join("filtered.vcf.gz");
    let filtered_tbi = out_dir.join("filtered.vcf.gz.tbi");
    atomic_write_bytes(&filtered_vcf, out.as_bytes())?;
    atomic_write_bytes(&filtered_tbi, b"tabix-index-placeholder\n")?;
    let filter_breakdown_json = out_dir.join("filter_breakdown.json");
    atomic_write_json(
        &filter_breakdown_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.filter_breakdown.v1",
            "expression": expression,
            "counts": tag_counts
        }),
    )?;
    let filter_breakdown_tsv = out_dir.join("filter_breakdown.tsv");
    let mut rows = String::from("tag\tcount\n");
    for (tag, count) in &tag_counts {
        rows.push_str(&format!("{tag}\t{count}\n"));
    }
    atomic_write_bytes(&filter_breakdown_tsv, rows.as_bytes())?;
    Ok(FilterStageOutputs {
        filtered_vcf,
        filtered_tbi,
        filter_breakdown_json,
        filter_breakdown_tsv,
    })
}

