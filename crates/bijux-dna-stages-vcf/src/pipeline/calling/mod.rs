mod damage_and_propagation;
mod types;
mod variant_records;

use super::*;
pub use damage_and_propagation::*;
pub use types::*;
pub(crate) use variant_records::*;

fn resolve_reference_path(params: &VcfCallParams) -> Result<String> {
    if let Some(reference) = params.reference_fasta.as_deref() {
        let reference_path = Path::new(reference);
        if !reference_path.exists() {
            bail!(
                "call stage BAM flow reference_fasta does not exist: {}",
                reference_path.display()
            );
        }
        return Ok(reference.to_string());
    }
    let species = std::env::var("BIJUX_SPECIES_ID")
        .map_err(|_| anyhow!("call stage BAM flow requires reference_fasta or BIJUX_SPECIES_ID"))?;
    let build = std::env::var("BIJUX_BUILD_ID")
        .map_err(|_| anyhow!("call stage BAM flow requires reference_fasta or BIJUX_BUILD_ID"))?;
    let bundle = bijux_dna_db_ref::resolve_reference_bundle(&species, &build)
        .map_err(|err| anyhow!("db-ref resolution failed for {species}:{build}: {err}"))?;
    Ok(bundle.fasta)
}

fn ensure_bam_prerequisites(input_bam: &Path, params: &VcfCallParams) -> Result<()> {
    if input_bam.extension().and_then(|x| x.to_str()) != Some("bam") {
        bail!("call stage BAM flow requires .bam input: {}", input_bam.display());
    }
    let bai = input_bam.with_extension("bam.bai");
    if !bai.exists() {
        bail!("call stage BAM flow requires BAM index: {}", bai.display());
    }
    let reference = resolve_reference_path(params)?;
    if !Path::new(&reference).exists() {
        bail!("call stage BAM flow reference does not exist: {}", reference);
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
    let reference = resolve_reference_path(params)?;
    let mpileup_bcf =
        out_vcf.parent().ok_or_else(|| anyhow!("output path has no parent"))?.join("mpileup.bcf");
    let min_map_q = params.min_mapping_quality.to_string();
    let min_base_q = params.min_base_quality.to_string();
    let mpileup_out =
        mpileup_bcf.to_str().ok_or_else(|| anyhow!("non-utf8 mpileup output path"))?;
    let input_bam_s = input_bam.to_str().ok_or_else(|| anyhow!("non-utf8 input bam path"))?;
    let out_vcf_s = out_vcf.to_str().ok_or_else(|| anyhow!("non-utf8 output vcf path"))?;
    if include_gl_fields {
        run_bcftools_mpileup_likelihood_vcf(input_bam, out_vcf, params)?;
    } else {
        let mpileup_args = [
            "mpileup",
            "-Ob",
            "-f",
            reference.as_str(),
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

fn run_bcftools_mpileup_likelihood_vcf(
    input_bam: &Path,
    out_vcf: &Path,
    params: &VcfCallParams,
) -> Result<()> {
    ensure_bam_prerequisites(input_bam, params)?;
    let reference = resolve_reference_path(params)?;
    let out_dir = out_vcf.parent().ok_or_else(|| anyhow!("output path has no parent"))?;
    let mpileup_bcf = out_dir.join("mpileup.bcf");
    let mpileup_vcf = out_dir.join("mpileup_gl.vcf");
    let min_map_q = params.min_mapping_quality.to_string();
    let min_base_q = params.min_base_quality.to_string();
    let mpileup_out =
        mpileup_bcf.to_str().ok_or_else(|| anyhow!("non-utf8 mpileup output path"))?;
    let mpileup_vcf_out =
        mpileup_vcf.to_str().ok_or_else(|| anyhow!("non-utf8 mpileup VCF output path"))?;
    let input_bam_s = input_bam.to_str().ok_or_else(|| anyhow!("non-utf8 input bam path"))?;
    let mpileup_args = [
        "mpileup",
        "-a",
        "FORMAT/DP",
        "-Ob",
        "-f",
        reference.as_str(),
        "-q",
        min_map_q.as_str(),
        "-Q",
        min_base_q.as_str(),
        "-o",
        mpileup_out,
        input_bam_s,
    ];
    run_checked_command("bcftools", &mpileup_args)?;
    let view_args = ["view", "-Ov", "-o", mpileup_vcf_out, mpileup_out];
    run_checked_command("bcftools", &view_args)?;
    let _ = write_vcf_with_best_effort_index(
        out_vcf,
        &std::fs::read_to_string(&mpileup_vcf)?,
        "call_gl",
    )?;
    let _ = std::fs::remove_file(&mpileup_vcf);
    Ok(())
}

fn run_gatk_haplotype_caller(
    input_bam: &Path,
    out_vcf: &Path,
    params: &VcfCallParams,
) -> Result<()> {
    ensure_bam_prerequisites(input_bam, params)?;
    let reference = resolve_reference_path(params)?;
    let out_dir = out_vcf.parent().ok_or_else(|| anyhow!("output path has no parent"))?;
    let raw_vcf = out_dir.join("gatk.raw.vcf");
    let input_bam_s = input_bam.to_str().ok_or_else(|| anyhow!("non-utf8 input bam path"))?;
    let raw_vcf_s =
        raw_vcf.to_str().ok_or_else(|| anyhow!("non-utf8 temporary gatk output path"))?;
    let reference_s =
        Path::new(&reference).to_str().ok_or_else(|| anyhow!("non-utf8 reference path"))?;
    let output = std::process::Command::new("gatk")
        .args([
            "HaplotypeCaller",
            "-R",
            reference_s,
            "-I",
            input_bam_s,
            "-O",
            raw_vcf_s,
            "-ERC",
            "GVCF",
            "--min-base-quality-score",
            &params.min_base_quality.to_string(),
        ])
        .output()
        .map_err(|err| anyhow!("gatk invocation failed: {err}"))?;
    if !output.status.success() {
        bail!("gatk HaplotypeCaller failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let _ = crate::vcf_io::vcf_index_bgzip_tabix(&raw_vcf, out_vcf)?;
    let _ = std::fs::remove_file(&raw_vcf);
    Ok(())
}

fn try_run_angsd_gl_from_bam(
    input_bam: &Path,
    out_dir: &Path,
    params: &VcfCallParams,
) -> Result<bool> {
    ensure_bam_prerequisites(input_bam, params)?;
    let reference = resolve_reference_path(params)?;
    let min_map_q = params.min_mapping_quality.to_string();
    let min_base_q = params.min_base_quality.to_string();
    let out_prefix = out_dir.join("angsd_gl");
    let out_prefix_s =
        out_prefix.to_str().ok_or_else(|| anyhow!("non-utf8 angsd output prefix"))?;
    let input_bam_s = input_bam.to_str().ok_or_else(|| anyhow!("non-utf8 input bam path"))?;
    let args = [
        "-i",
        input_bam_s,
        "-ref",
        reference.as_str(),
        "-GL",
        "2",
        "-doGlf",
        "2",
        "-doMajorMinor",
        "1",
        "-doMaf",
        "1",
        "-minMapQ",
        min_map_q.as_str(),
        "-minQ",
        min_base_q.as_str(),
        "-out",
        out_prefix_s,
    ];
    let output = std::process::Command::new("angsd").args(args).output();
    let log_path = out_dir.join("angsd_call_gl.log");
    match output {
        Ok(result) if result.status.success() => {
            let mut log = String::from("status=ok\n");
            if !result.stdout.is_empty() {
                log.push_str("stdout:\n");
                log.push_str(&String::from_utf8_lossy(&result.stdout));
                log.push('\n');
            }
            if !result.stderr.is_empty() {
                log.push_str("stderr:\n");
                log.push_str(&String::from_utf8_lossy(&result.stderr));
                log.push('\n');
            }
            atomic_write_bytes(&log_path, log.as_bytes())?;
            Ok(true)
        }
        Ok(result) => {
            let mut log = format!("status=failed\nexit={}\n", result.status);
            if !result.stderr.is_empty() {
                log.push_str("stderr:\n");
                log.push_str(&String::from_utf8_lossy(&result.stderr));
                log.push('\n');
            }
            atomic_write_bytes(&log_path, log.as_bytes())?;
            Ok(false)
        }
        Err(err) => {
            atomic_write_bytes(
                &log_path,
                format!("status=missing_or_failed_to_launch\nerror={err}\n").as_bytes(),
            )?;
            Ok(false)
        }
    }
}

fn write_vcf_with_best_effort_index(
    out_vcf: &Path,
    payload: &str,
    stage_label: &str,
) -> Result<PathBuf> {
    let plain_vcf = out_vcf
        .parent()
        .ok_or_else(|| anyhow!("{stage_label}: output path has no parent"))?
        .join(format!("{stage_label}.tmp.vcf"));
    atomic_write_bytes(&plain_vcf, payload.as_bytes())?;
    let out_tbi = crate::vcf_io::vcf_index_bgzip_tabix(&plain_vcf, out_vcf).map_err(|err| {
        anyhow!("{stage_label}: bgzip+tabix failed for {}: {err}", out_vcf.display())
    })?;
    let _ = std::fs::remove_file(&plain_vcf);
    Ok(out_tbi)
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
    let raw = read_vcf_text(output_vcf)?;
    let mut depth = std::collections::BTreeMap::<String, u64>::new();
    let mut ct_ga_total = 0_u64;
    let mut ct_ga_pass = 0_u64;
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let reference = fields[3].to_ascii_uppercase();
        let alternate = fields[4].to_ascii_uppercase();
        let ct_or_ga =
            (reference == "C" && alternate == "T") || (reference == "G" && alternate == "A");
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
        bail!("call stage contract violation: missing tabix index for {}", output_vcf.display());
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
    let raw = read_vcf_text(input_vcf)?;
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
    let _ = write_vcf_with_best_effort_index(&out_vcf, &normalized, "call_gl")?;
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
    if params.caller == "angsd" {
        let _ = try_run_angsd_gl_from_bam(input_bam, out_dir, params)?;
    }
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
        if params.caller == "gatk" {
            run_gatk_haplotype_caller(input_vcf, &out_vcf, params)?;
        } else {
            run_bcftools_mpileup_call(input_vcf, &out_vcf, params, false)?;
        }
        return write_call_outputs(out_dir, CallStageKind::Diploid, input_vcf, &out_vcf, params);
    }
    let raw = read_vcf_text(input_vcf)?;
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
    let normalized = normalize_header_sample_order(&out);
    let _ = write_vcf_with_best_effort_index(&out_vcf, &normalized, "call_diploid")?;
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
        let raw = read_vcf_text(&source_vcf)?;
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
        let normalized = normalize_header_sample_order(&out);
        let _ = write_vcf_with_best_effort_index(&out_vcf, &normalized, "call_pseudohaploid")?;
        return write_call_outputs(
            out_dir,
            CallStageKind::Pseudohaploid,
            input_vcf,
            &out_vcf,
            params,
        );
    }
    let raw = read_vcf_text(input_vcf)?;
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
    let normalized = normalize_header_sample_order(&out);
    let _ = write_vcf_with_best_effort_index(&out_vcf, &normalized, "call_pseudohaploid")?;
    write_call_outputs(out_dir, CallStageKind::Pseudohaploid, input_vcf, &out_vcf, params)
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
    pub filter_explain_json: PathBuf,
}

/// # Errors
/// Returns an error if filter stage outputs cannot be materialized.
pub fn run_filter_stage_real(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfFilterParams,
) -> Result<FilterStageOutputs> {
    let raw = read_vcf_text(input_vcf)?;
    let mut out = String::new();
    let mut kept = 0u64;
    let mut tag_counts = std::collections::BTreeMap::<String, u64>::new();
    let thresholds = load_imputation_qc_thresholds();
    let maf_min = *thresholds.get("vcf_filter_maf_min").unwrap_or(&0.01);
    let sample_missingness_max =
        *thresholds.get("vcf_filter_sample_missingness_max").unwrap_or(&0.20);
    let dp_min = *thresholds.get("vcf_filter_dp_min").unwrap_or(&8.0);
    let mq_min = *thresholds.get("vcf_filter_mq_min").unwrap_or(&30.0);
    let strand_bias_max = *thresholds.get("vcf_filter_strand_bias_max").unwrap_or(&60.0);
    let expression = format!(
        "QUAL>={:.3} && DP>={:.3} && MQ>={:.3} && FS<={:.3} && F_MISSING<={:.3} && (AF>={:.3} || AF missing)",
        params.min_qual, dp_min, mq_min, strand_bias_max, sample_missingness_max, maf_min
    );
    let mut total_records = 0_u64;
    for line in raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            total_records += 1;
            let qual = fields[5].parse::<f64>().unwrap_or(0.0);
            let af = parse_af_from_info(fields[7]);
            let dp = parse_depth_from_info(fields[7]).map_or(0.0, f64::from);
            let mq = parse_info_value_f64(fields[7], "MQ").unwrap_or(f64::INFINITY);
            let strand_bias = parse_info_value_f64(fields[7], "FS")
                .or_else(|| parse_info_value_f64(fields[7], "SOR"))
                .unwrap_or(0.0);
            let f_missing = if fields.len() > 9 {
                genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0)
            } else {
                0.0
            };
            let mut reasons = Vec::<&str>::new();
            if qual < params.min_qual {
                reasons.push("LOWQUAL");
            }
            if dp < dp_min {
                reasons.push("LOW_DP");
            }
            if mq < mq_min {
                reasons.push("LOW_MQ");
            }
            if strand_bias > strand_bias_max {
                reasons.push("STRAND_BIAS");
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
            row[6] = if reasons.is_empty() { "PASS".to_string() } else { reasons.join(";") };
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
        return Err(anyhow!("vcf.filter removed all variants in production_profile mode"));
    }
    if params.production_profile && total_records > 0 {
        let retention = kept as f64 / total_records as f64;
        let fail = *thresholds.get("vcf_filter_retention_fail").unwrap_or(&0.20);
        if retention < fail {
            bail!("vcf.filter production gate failed: retention below fail threshold");
        }
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let filtered_vcf = out_dir.join("filtered.vcf.gz");
    let normalized = normalize_header_sample_order(&out);
    let filtered_tbi = write_vcf_with_best_effort_index(&filtered_vcf, &normalized, "vcf_filter")?;
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
    let filter_explain_json = out_dir.join("filter_explain.json");
    atomic_write_json(
        &filter_explain_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.filter_explain.v1",
            "filter_expression": expression,
            "filter_scope": {
                "site_level": ["min_qual", "min_depth", "min_mapping_quality", "strand_bias_max", "maf_min"],
                "sample_level": ["sample_missingness_max"],
                "damage_aware_filters": [],
                "output_subset": if params.require_pass { "pass_only" } else { "retain_tagged_records" }
            },
            "thresholds": {
                "min_qual": params.min_qual,
                "min_depth": dp_min,
                "min_mapping_quality": mq_min,
                "strand_bias_max": strand_bias_max,
                "sample_missingness_max": sample_missingness_max,
                "maf_min": maf_min
            },
            "normalization_enabled": params.normalize,
            "counts": tag_counts
        }),
    )?;
    Ok(FilterStageOutputs {
        filtered_vcf,
        filtered_tbi,
        filter_breakdown_json,
        filter_breakdown_tsv,
        filter_explain_json,
    })
}
