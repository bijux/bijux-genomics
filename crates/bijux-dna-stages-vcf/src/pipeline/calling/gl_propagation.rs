use super::*;

#[derive(Debug, Clone)]
pub struct GlPropagationStageParams {
    pub require_gl_or_pl: bool,
    pub expected_ploidy: Option<u8>,
    pub emit_bcf: bool,
}

impl GlPropagationStageParams {
    #[must_use]
    pub fn recommended() -> Self {
        Self { require_gl_or_pl: true, expected_ploidy: Some(2), emit_bcf: true }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GlPropagationOutputs {
    pub normalized_vcf: PathBuf,
    pub normalized_tbi: PathBuf,
    pub normalized_bcf: Option<PathBuf>,
    pub normalized_bcf_csi: Option<PathBuf>,
    pub gl_propagation_report_json: PathBuf,
}

/// # Errors
/// Returns an error if GL propagation contracts are violated.
pub fn run_gl_propagation_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &GlPropagationStageParams,
) -> Result<GlPropagationOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut output_lines = Vec::<String>::new();
    let mut has_gl_or_pl = false;
    let mut allele_reordered = 0_u64;
    let mut ploidy_mismatch = 0_u64;
    let mut missing_normalized = 0_u64;
    let mut records = 0_u64;

    for line in raw.lines() {
        if line.starts_with('#') {
            output_lines.push(line.to_string());
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        records += 1;
        let mut row = fields.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
        if row.len() > 8 {
            let fmt = row[8].clone();
            if format_has_token(&fmt, &["GL", "PL"]) {
                has_gl_or_pl = true;
            }
            if row.len() > 9 {
                let before = row[9].clone();
                row[9] = normalize_sample_fields(&fmt, &row[9]);
                if row[9] != before {
                    missing_normalized += 1;
                }
                if let Some(expected) = params.expected_ploidy {
                    let keys = fmt.split(':').collect::<Vec<_>>();
                    if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
                        let vals = row[9].split(':').collect::<Vec<_>>();
                        if let Some(gt) = vals.get(gt_idx) {
                            let observed = gt.split(['/', '|']).count() as u8;
                            if !gt.contains('.') && observed != expected {
                                ploidy_mismatch += 1;
                            }
                        }
                    }
                }
            }
        }
        if row.len() > 4 {
            let mut alts = row[4].split(',').map(str::to_string).collect::<Vec<_>>();
            let original = alts.clone();
            alts.sort();
            if alts != original {
                if row.len() > 8 && format_has_token(&row[8], &["GL", "PL"]) {
                    bail!("vcf.gl_propagation refusal: allele ordering mismatch with GL/PL fields");
                }
                row[4] = alts.join(",");
                allele_reordered += 1;
            }
        }
        output_lines.push(row.join("\t"));
    }
    if let Some(chrom_idx) = output_lines.iter().position(|line| line.starts_with("#CHROM\t")) {
        let has_gt = output_lines.iter().any(|line| line.starts_with("##FORMAT=<ID=GT,"));
        let has_gl = output_lines.iter().any(|line| line.starts_with("##FORMAT=<ID=GL,"));
        let has_pl = output_lines.iter().any(|line| line.starts_with("##FORMAT=<ID=PL,"));
        if !has_gt {
            output_lines.insert(
                chrom_idx,
                "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">".to_string(),
            );
        }
        let insert_at =
            output_lines.iter().position(|line| line.starts_with("#CHROM\t")).unwrap_or(chrom_idx);
        if !has_gl {
            output_lines.insert(
                insert_at,
                "##FORMAT=<ID=GL,Number=G,Type=Float,Description=\"Genotype Likelihood\">"
                    .to_string(),
            );
        }
        let insert_at =
            output_lines.iter().position(|line| line.starts_with("#CHROM\t")).unwrap_or(chrom_idx);
        if !has_pl {
            output_lines.insert(
                insert_at,
                "##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled Genotype Likelihood\">".to_string(),
            );
        }
    }
    if params.require_gl_or_pl && !has_gl_or_pl {
        bail!("vcf.gl_propagation requires GL/PL in FORMAT for downstream compatibility");
    }
    if ploidy_mismatch > 0 {
        bail!("vcf.gl_propagation refusal: ploidy mismatch detected in GT fields");
    }

    let normalized_vcf = out_dir.join("gl_normalized.vcf.gz");
    let normalized_payload =
        output_lines.join("\n") + if output_lines.is_empty() { "" } else { "\n" };
    let normalized_tbi =
        write_vcf_with_best_effort_index(&normalized_vcf, &normalized_payload, "gl_propagation")?;

    let (normalized_bcf, normalized_bcf_csi) = if params.emit_bcf {
        let bcf = out_dir.join("gl_normalized.bcf");
        let csi = out_dir.join("gl_normalized.bcf.csi");
        let normalized_vcf_s =
            normalized_vcf.to_str().ok_or_else(|| anyhow!("non-utf8 gl normalized vcf path"))?;
        let bcf_s = bcf.to_str().ok_or_else(|| anyhow!("non-utf8 gl normalized bcf path"))?;
        crate::engine::execution::run_checked_command(
            "bcftools",
            ["view", "-Ob", "-o", bcf_s, normalized_vcf_s],
            None,
        )?;
        crate::engine::execution::run_checked_command("bcftools", ["index", "-f", bcf_s], None)?;
        if !csi.exists() {
            bail!("vcf.gl_propagation contract violation: missing BCF index {}", csi.display());
        }
        (Some(bcf), Some(csi))
    } else {
        (None, None)
    };

    let gl_propagation_report_json = out_dir.join("gl_propagation_report.json");
    atomic_write_json(
        &gl_propagation_report_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.gl_propagation_report.v1",
            "records_seen": records,
            "has_gl_or_pl": has_gl_or_pl,
            "allele_reordered_records": allele_reordered,
            "missing_genotype_fields_normalized": missing_normalized,
            "expected_ploidy": params.expected_ploidy,
            "emit_bcf": params.emit_bcf
        }),
    )?;

    Ok(GlPropagationOutputs {
        normalized_vcf,
        normalized_tbi,
        normalized_bcf,
        normalized_bcf_csi,
        gl_propagation_report_json,
    })
}
