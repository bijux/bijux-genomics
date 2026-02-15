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

fn run_checked_command(bin: &str, args: &[&str]) -> Result<()> {
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .map_err(|err| anyhow!("{bin} invocation failed: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{bin} failed: {stderr}");
    }
    Ok(())
}

fn read_vcf_text(path: &Path) -> Result<String> {
    if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x == "gz" || x == "bcf")
    {
        let output = std::process::Command::new("bcftools")
            .args(["view", &path.display().to_string()])
            .output()
            .map_err(|err| anyhow!("bcftools view invocation failed: {err}"))?;
        if !output.status.success() {
            bail!(
                "bcftools view failed while reading {}: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    Ok(std::fs::read_to_string(path)?)
}

/// # Errors
/// Returns an error if filter stage outputs cannot be materialized.
pub fn run_filter_stage_real(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfFilterParams,
) -> Result<FilterStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let source_vcfgz = if input_vcf
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x == "gz")
    {
        input_vcf.to_path_buf()
    } else {
        let plain_input = out_dir.join("filter_input.normalized.vcf");
        let normalized_input = out_dir.join("filter_input.normalized.vcf.gz");
        std::fs::copy(input_vcf, &plain_input)?;
        let _ = crate::vcf_io::vcf_index_bgzip_tabix(&plain_input, &normalized_input)?;
        normalized_input
    };
    let filtered_vcf = out_dir.join("filtered.vcf.gz");
    let filtered_tbi = out_dir.join("filtered.vcf.gz.tbi");
    let mut kept = 0u64;
    let mut tag_counts = std::collections::BTreeMap::<String, u64>::new();
    let maf_min = 0.01_f64;
    let sample_missingness_max = 0.20_f64;
    let expression = format!("QUAL<{:.3}", params.min_qual);
    let source_s = source_vcfgz
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 source vcf path"))?;
    let filtered_s = filtered_vcf
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 filtered vcf path"))?;
    run_checked_command(
        "bcftools",
        &[
            "filter",
            "-s",
            "LOWQUAL",
            "-e",
            &expression,
            "-Oz",
            "-o",
            filtered_s,
            source_s,
        ],
    )?;
    run_checked_command("tabix", &["-f", "-p", "vcf", filtered_s])?;

    if params.require_pass {
        let pass_only = out_dir.join("filtered.pass.vcf.gz");
        let pass_only_s = pass_only
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 filtered pass vcf path"))?;
        run_checked_command(
            "bcftools",
            &[
                "view",
                "-f",
                "PASS,.",
                "-Oz",
                "-o",
                pass_only_s,
                filtered_s,
            ],
        )?;
        run_checked_command("tabix", &["-f", "-p", "vcf", pass_only_s])?;
        std::fs::rename(pass_only, &filtered_vcf)?;
        let pass_tbi = PathBuf::from(format!("{}.tbi", filtered_vcf.display()));
        if pass_tbi.exists() {
            let _ = std::fs::rename(&pass_tbi, &filtered_tbi);
        }
    }

    let raw = read_vcf_text(&filtered_vcf)?;
    let mut total_records = 0_u64;
    for line in raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            total_records += 1;
            let af = parse_af_from_info(fields[7]);
            let f_missing = if fields.len() > 9 {
                genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0)
            } else {
                0.0
            };
            let mut reasons = Vec::<&str>::new();
            if fields[6].contains("LOWQUAL") {
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
            kept += 1;
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
    if !filtered_tbi.exists() {
        bail!(
            "vcf.filter contract violation: missing tabix index for {}",
            filtered_vcf.display()
        );
    }
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
