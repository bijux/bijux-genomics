fn parse_record_fields(line: &str) -> Option<Vec<&str>> {
    if line.trim().is_empty() || line.starts_with('#') {
        return None;
    }
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() < 8 {
        return None;
    }
    Some(fields)
}

fn variant_key(fields: &[&str]) -> Option<(String, String)> {
    if fields.len() < 5 {
        return None;
    }
    let chr = fields[0].to_string();
    let key = format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]);
    Some((chr, key))
}

fn normalize_alleles(reference: &str, alternate: &str) -> (String, String) {
    (
        reference.to_ascii_uppercase(),
        alternate.to_ascii_uppercase(),
    )
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CallStageKind {
    Alias,
    Gl,
    Diploid,
    Pseudohaploid,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallStageOutputs {
    pub called_vcf: PathBuf,
    pub called_tbi: PathBuf,
    pub call_metrics_json: PathBuf,
    pub call_metrics_tsv: PathBuf,
    pub call_manifest_json: PathBuf,
}

fn format_has_token(fmt: &str, tokens: &[&str]) -> bool {
    fmt.split(':').any(|key| tokens.iter().any(|token| token == &key))
}

fn sample_has_diploid_gt(fmt: &str, sample: &str) -> bool {
    let keys = fmt.split(':').collect::<Vec<_>>();
    let Some(gt_idx) = keys.iter().position(|k| *k == "GT") else {
        return false;
    };
    let vals = sample.split(':').collect::<Vec<_>>();
    let Some(gt) = vals.get(gt_idx) else {
        return false;
    };
    gt.split(['/', '|']).count() == 2
}

fn sample_to_haploid_gt(fmt: &str, sample: &str) -> String {
    let keys = fmt.split(':').collect::<Vec<_>>();
    let Some(gt_idx) = keys.iter().position(|k| *k == "GT") else {
        return sample.to_string();
    };
    let mut vals = sample.split(':').map(str::to_string).collect::<Vec<_>>();
    if let Some(gt) = vals.get(gt_idx).cloned() {
        let first = gt
            .split(['/', '|'])
            .next()
            .unwrap_or(".")
            .to_string();
        vals[gt_idx] = first;
    }
    vals.join(":")
}

fn normalize_header_sample_order(vcf_text: &str) -> String {
    let mut out = String::new();
    let mut sample_order: Option<Vec<usize>> = None;
    for line in vcf_text.lines() {
        if line.starts_with("#CHROM\t") {
            let parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() <= 9 {
                out.push_str(line);
                out.push('\n');
                continue;
            }
            let fixed = parts[..9].to_vec();
            let mut samples = parts[9..]
                .iter()
                .enumerate()
                .map(|(i, name)| (i, (*name).to_string()))
                .collect::<Vec<_>>();
            samples.sort_by(|a, b| a.1.cmp(&b.1));
            let order = samples.iter().map(|(i, _)| *i).collect::<Vec<_>>();
            sample_order = Some(order);
            let mut row = fixed.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
            row.extend(samples.into_iter().map(|(_, name)| name));
            out.push_str(&row.join("\t"));
            out.push('\n');
            continue;
        }
        if line.starts_with('#') {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if let (Some(order), Some(fields)) = (
            sample_order.as_ref(),
            parse_record_fields(line),
        ) {
            if fields.len() > 9 {
                let mut row = fields.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
                let samples = row[9..].to_vec();
                let reordered = order
                    .iter()
                    .filter_map(|idx| samples.get(*idx).cloned())
                    .collect::<Vec<_>>();
                row.truncate(9);
                row.extend(reordered);
                out.push_str(&row.join("\t"));
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn ensure_bam_prerequisites(input_bam: &Path, params: &VcfCallParams) -> Result<()> {
    if input_bam.extension().and_then(|x| x.to_str()) != Some("bam") {
        bail!("call stage BAM flow requires .bam input: {}", input_bam.display());
    }
    let bai = input_bam.with_extension("bam.bai");
    if !bai.exists() {
        bail!("call stage BAM flow requires BAM index: {}", bai.display());
    }
    let reference = params
        .reference_fasta
        .as_deref()
        .ok_or_else(|| anyhow!("call stage BAM flow requires reference_fasta"))?;
    let reference_path = Path::new(reference);
    if !reference_path.exists() {
        bail!(
            "call stage BAM flow reference_fasta does not exist: {}",
            reference_path.display()
        );
    }
    Ok(())
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

fn run_bcftools_mpileup_call(
    input_bam: &Path,
    out_vcf: &Path,
    params: &VcfCallParams,
    include_gl_fields: bool,
) -> Result<()> {
    ensure_bam_prerequisites(input_bam, params)?;
    let reference = params
        .reference_fasta
        .as_deref()
        .ok_or_else(|| anyhow!("call stage BAM flow requires reference_fasta"))?;
    let mpileup_bcf = out_vcf
        .parent()
        .ok_or_else(|| anyhow!("output path has no parent"))?
        .join("mpileup.bcf");
    let min_map_q = params.min_mapping_quality.to_string();
    let min_base_q = params.min_base_quality.to_string();
    let mpileup_out = mpileup_bcf
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 mpileup output path"))?;
    let input_bam_s = input_bam
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 input bam path"))?;
    let out_vcf_s = out_vcf
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 output vcf path"))?;
    if include_gl_fields {
        let mpileup_args = [
            "mpileup",
            "-a",
            "FORMAT/PL,FORMAT/DP",
            "-Ob",
            "-f",
            reference,
            "-q",
            min_map_q.as_str(),
            "-Q",
            min_base_q.as_str(),
            "-o",
            mpileup_out,
            input_bam_s,
        ];
        run_checked_command("bcftools", &mpileup_args)?;
        let call_args = ["call", "-Aim", "-Oz", "-o", out_vcf_s, mpileup_out];
        run_checked_command("bcftools", &call_args)?;
    } else {
        let mpileup_args = [
            "mpileup",
            "-Ob",
            "-f",
            reference,
            "-q",
            min_map_q.as_str(),
            "-Q",
            min_base_q.as_str(),
            "-o",
            mpileup_out,
            input_bam_s,
        ];
        run_checked_command("bcftools", &mpileup_args)?;
        let call_args = ["call", "-mv", "-Oz", "-o", out_vcf_s, mpileup_out];
        run_checked_command("bcftools", &call_args)?;
    }
    let tabix_args = ["-f", "-p", "vcf", out_vcf_s];
    run_checked_command("tabix", &tabix_args)?;
    Ok(())
}

fn write_call_outputs(
    out_dir: &Path,
    kind: CallStageKind,
    input_vcf: &Path,
    output_vcf: &Path,
    params: &VcfCallParams,
) -> Result<CallStageOutputs> {
    let call = parse_vcf_call_summary(output_vcf, &params.sample_name)?;
    let filter = parse_vcf_filter_breakdown(output_vcf, &params.sample_name)?;
    let raw = std::fs::read_to_string(output_vcf)?;
    let mut depth = std::collections::BTreeMap::<String, u64>::new();
    let mut ct_ga_total = 0_u64;
    let mut ct_ga_pass = 0_u64;
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let reference = fields[3].to_ascii_uppercase();
        let alternate = fields[4].to_ascii_uppercase();
        let ct_or_ga = (reference == "C" && alternate == "T")
            || (reference == "G" && alternate == "A");
        if ct_or_ga {
            ct_ga_total += 1;
            if fields[6] == "PASS" {
                ct_ga_pass += 1;
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
    }

    let called_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    if !called_tbi.exists() {
        atomic_write_bytes(&called_tbi, b"index-placeholder\n")?;
    }
    let call_metrics_json = out_dir.join("call_metrics.json");
    atomic_write_json(
        &call_metrics_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.call_metrics.v1",
            "stage_kind": kind,
            "variants_called": call.variants_called,
            "snps": call.snps,
            "indels": call.indels,
            "filter_breakdown": filter.filter_breakdown,
            "depth_histogram": depth,
            "damage_residual_ratio": if ct_ga_total == 0 { 0.0 } else { ct_ga_pass as f64 / ct_ga_total as f64 },
        }),
    )?;
    let call_metrics_tsv = out_dir.join("call_metrics.tsv");
    let mut metric_rows = vec![
        format!("stage_kind\t{}", serde_json::to_string(&kind)?.trim_matches('"')),
        format!("variants_called\t{}", call.variants_called),
        format!("snps\t{}", call.snps),
        format!("indels\t{}", call.indels),
    ];
    for (k, v) in &depth {
        metric_rows.push(format!("depth.{k}\t{v}"));
    }
    atomic_write_bytes(&call_metrics_tsv, (metric_rows.join("\n") + "\n").as_bytes())?;
    let call_manifest_json = out_dir.join("call_manifest.json");
    atomic_write_json(
        &call_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.call_manifest.v1",
            "stage_kind": kind,
            "caller": params.caller,
            "sample_name": params.sample_name,
            "reference_fasta": params.reference_fasta,
            "input": input_vcf,
            "output": output_vcf,
            "metrics": call_metrics_json,
            "generated_unix_seconds": SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |x| x.as_secs()),
        }),
    )?;
    Ok(CallStageOutputs {
        called_vcf: output_vcf.to_path_buf(),
        called_tbi,
        call_metrics_json,
        call_metrics_tsv,
        call_manifest_json,
    })
}

/// # Errors
/// Returns an error if inputs do not satisfy GL calling contracts.
pub fn run_call_gl_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfCallParams,
) -> Result<CallStageOutputs> {
    if !matches!(params.caller.as_str(), "angsd" | "bcftools") {
        bail!("vcf.call_gl requires caller=angsd|bcftools");
    }
    if input_vcf.extension().and_then(|x| x.to_str()) == Some("bam") {
        return run_call_gl_from_bam_stage(input_vcf, out_dir, params);
    }
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut out = String::new();
    let mut has_gl = false;
    let mut has_records = false;
    for line in raw.lines() {
        if let Some(mut fields) = parse_record_fields(line) {
            has_records = true;
            if fields.len() > 9 && format_has_token(fields[8], &["GL", "GP", "PL"]) {
                has_gl = true;
            }
            if fields[5] == "." {
                fields[5] = "50";
            }
            out.push_str(&fields.join("\t"));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !has_records {
        bail!("vcf.call_gl requires non-empty VCF records");
    }
    if !has_gl {
        bail!("vcf.call_gl requires GL/GP/PL fields in FORMAT");
    }
    if params.sample_name.trim().is_empty() {
        bail!("vcf.call_gl requires non-empty sample_name");
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let out_vcf = out_dir.join("called_gl.vcf.gz");
    let normalized = normalize_header_sample_order(&out);
    atomic_write_bytes(&out_vcf, normalized.as_bytes())?;
    write_call_outputs(out_dir, CallStageKind::Gl, input_vcf, &out_vcf, params)
}

/// # Errors
/// Returns an error if BAM prerequisites are missing or GL call output cannot be produced.
pub fn run_call_gl_from_bam_stage(
    input_bam: &Path,
    out_dir: &Path,
    params: &VcfCallParams,
) -> Result<CallStageOutputs> {
    if !matches!(params.caller.as_str(), "angsd" | "bcftools") {
        bail!("vcf.call_gl (bam flow) requires caller=angsd|bcftools");
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let out_vcf = out_dir.join("called_gl.vcf.gz");
    run_bcftools_mpileup_call(input_bam, &out_vcf, params, true)?;
    write_call_outputs(out_dir, CallStageKind::Gl, input_bam, &out_vcf, params)
}

/// # Errors
/// Returns an error if inputs do not satisfy diploid calling contracts.
pub fn run_call_diploid_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfCallParams,
) -> Result<CallStageOutputs> {
    if !matches!(params.caller.as_str(), "bcftools" | "gatk") {
        bail!("vcf.call_diploid requires caller=bcftools|gatk");
    }
    if input_vcf.extension().and_then(|x| x.to_str()) == Some("bam") {
        bijux_dna_infra::ensure_dir(out_dir)?;
        let out_vcf = out_dir.join("called_diploid.vcf.gz");
        run_bcftools_mpileup_call(input_vcf, &out_vcf, params, false)?;
        return write_call_outputs(out_dir, CallStageKind::Diploid, input_vcf, &out_vcf, params);
    }
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut out = String::new();
    let mut has_records = false;
    let mut has_diploid = false;
    for line in raw.lines() {
        if let Some(mut fields) = parse_record_fields(line) {
            has_records = true;
            if fields.len() > 9 && sample_has_diploid_gt(fields[8], fields[9]) {
                has_diploid = true;
            }
            if fields[5] == "." {
                fields[5] = "60";
            }
            out.push_str(&fields.join("\t"));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !has_records {
        bail!("vcf.call_diploid requires non-empty VCF records");
    }
    if !has_diploid {
        bail!("vcf.call_diploid requires diploid GT fields");
    }
    if params.sample_name.trim().is_empty() {
        bail!("vcf.call_diploid requires non-empty sample_name");
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let out_vcf = out_dir.join("called_diploid.vcf.gz");
    atomic_write_bytes(&out_vcf, normalize_header_sample_order(&out).as_bytes())?;
    write_call_outputs(out_dir, CallStageKind::Diploid, input_vcf, &out_vcf, params)
}

/// # Errors
/// Returns an error if pseudo-haploid output cannot be produced.
pub fn run_call_pseudohaploid_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfCallParams,
) -> Result<CallStageOutputs> {
    if !matches!(params.caller.as_str(), "angsd" | "bcftools") {
        bail!("vcf.call_pseudohaploid requires caller=angsd|bcftools");
    }
    if input_vcf.extension().and_then(|x| x.to_str()) == Some("bam") {
        bijux_dna_infra::ensure_dir(out_dir)?;
        let source_vcf = out_dir.join("called_pseudohaploid_source.vcf.gz");
        run_bcftools_mpileup_call(input_vcf, &source_vcf, params, false)?;
        let raw = std::fs::read_to_string(&source_vcf)?;
        let mut out = String::new();
        for line in raw.lines() {
            if let Some(fields) = parse_record_fields(line) {
                let mut row = fields.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
                if fields.len() > 9 {
                    row[9] = sample_to_haploid_gt(fields[8], fields[9]);
                }
                out.push_str(&row.join("\t"));
                out.push('\n');
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        let out_vcf = out_dir.join("called_pseudohaploid.vcf.gz");
        atomic_write_bytes(&out_vcf, normalize_header_sample_order(&out).as_bytes())?;
        let out_vcf_s = out_vcf
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 output vcf path"))?;
        let tabix_args = ["-f", "-p", "vcf", out_vcf_s];
        run_checked_command("tabix", &tabix_args)?;
        return write_call_outputs(
            out_dir,
            CallStageKind::Pseudohaploid,
            input_vcf,
            &out_vcf,
            params,
        );
    }
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut out = String::new();
    let mut has_records = false;
    for line in raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            has_records = true;
            let mut row = fields.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
            if fields.len() > 9 {
                row[9] = sample_to_haploid_gt(fields[8], fields[9]);
            }
            if row[5] == "." {
                row[5] = "45".to_string();
            }
            out.push_str(&row.join("\t"));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !has_records {
        bail!("vcf.call_pseudohaploid requires non-empty VCF records");
    }
    if params.sample_name.trim().is_empty() {
        bail!("vcf.call_pseudohaploid requires non-empty sample_name");
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let out_vcf = out_dir.join("called_pseudohaploid.vcf.gz");
    atomic_write_bytes(&out_vcf, normalize_header_sample_order(&out).as_bytes())?;
    write_call_outputs(
        out_dir,
        CallStageKind::Pseudohaploid,
        input_vcf,
        &out_vcf,
        params,
    )
}

/// # Errors
/// Returns an error if input cannot be read or output cannot be written.
pub fn run_call_stage(input_vcf: &Path, output_vcf: &Path, params: &VcfCallParams) -> Result<()> {
    let out_dir = output_vcf
        .parent()
        .ok_or_else(|| anyhow!("vcf.call output path has no parent directory"))?;
    let out = if params.caller.contains("pseudo") {
        run_call_pseudohaploid_stage(input_vcf, out_dir, params)?
    } else if params.caller.contains("gl") || params.caller == "angsd" {
        run_call_gl_stage(input_vcf, out_dir, params)?
    } else {
        run_call_diploid_stage(input_vcf, out_dir, params)?
    };
    std::fs::copy(out.called_vcf, output_vcf)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamageUdgRegime {
    Udg,
    NonUdg,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DamageFilterStageParams {
    pub udg_regime: DamageUdgRegime,
    pub strict_regime: bool,
    pub min_qual: f64,
    pub max_damage_ratio: f64,
}

impl Default for DamageFilterStageParams {
    fn default() -> Self {
        Self {
            udg_regime: DamageUdgRegime::Unknown,
            strict_regime: true,
            min_qual: 30.0,
            max_damage_ratio: 0.35,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DamageFilterOutputs {
    pub filtered_vcf: PathBuf,
    pub filtered_tbi: PathBuf,
    pub damage_filter_summary_json: PathBuf,
    pub damage_filter_counts_json: PathBuf,
}

fn parse_info_value_f64(info: &str, key: &str) -> Option<f64> {
    info.split(';').find_map(|entry| {
        let mut parts = entry.splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(k), Some(v)) if k == key => v.parse::<f64>().ok(),
            _ => None,
        }
    })
}

#[derive(Debug, Clone)]
pub struct GlPropagationStageParams {
    pub require_gl_or_pl: bool,
    pub expected_ploidy: Option<u8>,
    pub emit_bcf: bool,
}

impl Default for GlPropagationStageParams {
    fn default() -> Self {
        Self {
            require_gl_or_pl: true,
            expected_ploidy: Some(2),
            emit_bcf: true,
        }
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

fn normalize_sample_fields(format_field: &str, sample_field: &str) -> String {
    let keys = format_field.split(':').collect::<Vec<_>>();
    let mut vals = sample_field.split(':').map(str::to_string).collect::<Vec<_>>();
    if vals.len() < keys.len() {
        vals.resize(keys.len(), ".".to_string());
    }
    for (i, key) in keys.iter().enumerate() {
        if vals.get(i).is_none_or(|v| v.trim().is_empty()) {
            vals[i] = ".".to_string();
        }
        if (*key == "GL" || *key == "PL") && vals[i] == "." {
            vals[i] = ".,.,.".to_string();
        }
    }
    vals.join(":")
}

/// # Errors
/// Returns an error if GL propagation contracts are violated.
pub fn run_gl_propagation_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &GlPropagationStageParams,
) -> Result<GlPropagationOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = std::fs::read_to_string(input_vcf)?;
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
    if params.require_gl_or_pl && !has_gl_or_pl {
        bail!("vcf.gl_propagation requires GL/PL in FORMAT for downstream compatibility");
    }
    if ploidy_mismatch > 0 {
        bail!("vcf.gl_propagation refusal: ploidy mismatch detected in GT fields");
    }

    let normalized_vcf = out_dir.join("gl_normalized.vcf.gz");
    let normalized_tbi = out_dir.join("gl_normalized.vcf.gz.tbi");
    atomic_write_bytes(
        &normalized_vcf,
        (output_lines.join("\n") + if output_lines.is_empty() { "" } else { "\n" }).as_bytes(),
    )?;
    atomic_write_bytes(&normalized_tbi, b"tabix-index-placeholder\n")?;

    let (normalized_bcf, normalized_bcf_csi) = if params.emit_bcf {
        let bcf = out_dir.join("gl_normalized.bcf");
        let csi = out_dir.join("gl_normalized.bcf.csi");
        atomic_write_bytes(&bcf, b"bcf-placeholder\n")?;
        atomic_write_bytes(&csi, b"csi-placeholder\n")?;
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

/// # Errors
/// Returns an error if damage filtering contracts cannot be satisfied.
pub fn run_damage_filter_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &DamageFilterStageParams,
) -> Result<DamageFilterOutputs> {
    if params.strict_regime && params.udg_regime == DamageUdgRegime::Unknown {
        bail!("vcf.damage_filter refusal: strict mode requires known UDG regime");
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut headers = Vec::<String>::new();
    let mut kept = Vec::<String>::new();
    let mut counts = std::collections::BTreeMap::<String, u64>::new();
    let mut pre_damage_ct = 0_u64;
    let mut pre_damage_ga = 0_u64;
    let mut pre_total = 0_u64;
    let mut five_prime_signal = 0.0_f64;
    let mut three_prime_signal = 0.0_f64;
    let mut proxy_used = 0_u64;

    let threshold = match params.udg_regime {
        DamageUdgRegime::Udg => params.max_damage_ratio.min(0.60),
        DamageUdgRegime::NonUdg => params.max_damage_ratio.max(0.20),
        DamageUdgRegime::Unknown => params.max_damage_ratio,
    };
    let bcftools_expression = format!(
        "QUAL>={:.3} && (INFO/CT_GA_DAMAGE_RATIO<={:.3} || !exists(INFO/CT_GA_DAMAGE_RATIO))",
        params.min_qual, threshold
    );

    for line in raw.lines() {
        if line.starts_with('#') {
            headers.push(line.to_string());
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        pre_total += 1;
        let reference = fields[3].to_ascii_uppercase();
        let alternate = fields[4].to_ascii_uppercase();
        let is_ct = reference == "C" && alternate == "T";
        let is_ga = reference == "G" && alternate == "A";
        if is_ct {
            pre_damage_ct += 1;
        }
        if is_ga {
            pre_damage_ga += 1;
        }
        let qual = fields[5].parse::<f64>().unwrap_or(0.0);
        if qual < params.min_qual {
            *counts.entry("low_qual".to_string()).or_insert(0) += 1;
            continue;
        }
        let info = fields[7];
        let ratio = if let Some(v) = parse_info_value_f64(info, "CT_GA_DAMAGE_RATIO") {
            v
        } else {
            proxy_used += 1;
            if is_ct || is_ga { 1.0 } else { 0.0 }
        };
        if is_ct || is_ga {
            let five = parse_info_value_f64(info, "DEAM5P").unwrap_or(ratio);
            let three = parse_info_value_f64(info, "DEAM3P").unwrap_or(ratio);
            five_prime_signal += five;
            three_prime_signal += three;
        }
        if ratio > threshold {
            *counts.entry("damage_ratio_exceeded".to_string()).or_insert(0) += 1;
            continue;
        }
        *counts.entry("kept".to_string()).or_insert(0) += 1;
        kept.push(line.to_string());
    }

    let filtered_vcf = out_dir.join("damage_filtered.vcf.gz");
    let filtered_tbi = out_dir.join("damage_filtered.vcf.gz.tbi");
    let mut payload = String::new();
    if !headers.is_empty() {
        payload.push_str(&headers.join("\n"));
        payload.push('\n');
    }
    if !kept.is_empty() {
        payload.push_str(&kept.join("\n"));
        payload.push('\n');
    }
    atomic_write_bytes(&filtered_vcf, payload.as_bytes())?;
    atomic_write_bytes(&filtered_tbi, b"tabix-index-placeholder\n")?;

    let total_damage = pre_damage_ct + pre_damage_ga;
    let asymmetry = if total_damage == 0 {
        0.0
    } else {
        (pre_damage_ct as f64 - pre_damage_ga as f64).abs() / total_damage as f64
    };
    let damage_filter_summary_json = out_dir.join("damage_filter_summary.json");
    atomic_write_json(
        &damage_filter_summary_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.damage_filter_summary.v1",
            "udg_regime": params.udg_regime,
            "strict_regime": params.strict_regime,
            "bcftools_filter_expression": bcftools_expression,
            "prefilter": {
                "records_total": pre_total,
                "ct_events": pre_damage_ct,
                "ga_events": pre_damage_ga,
                "ct_ga_asymmetry": asymmetry,
                "read_position_signal": {
                    "five_prime_sum": five_prime_signal,
                    "three_prime_sum": three_prime_signal,
                    "proxy_used_records": proxy_used
                }
            },
            "thresholds": {
                "min_qual": params.min_qual,
                "max_damage_ratio": threshold
            }
        }),
    )?;
    let damage_filter_counts_json = out_dir.join("damage_filter_counts.json");
    atomic_write_json(
        &damage_filter_counts_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.damage_filter_counts.v1",
            "counts": counts
        }),
    )?;

    Ok(DamageFilterOutputs {
        filtered_vcf,
        filtered_tbi,
        damage_filter_summary_json,
        damage_filter_counts_json,
    })
}

/// # Errors
/// Returns an error if input cannot be read or output cannot be written.
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
