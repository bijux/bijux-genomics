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
    let mut vals = sample.split(':').map(str::to_string).collect::<Vec<_>>();
    if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
        if let Some(gt) = vals.get(gt_idx).cloned() {
            let first = gt.split(['/', '|']).next().unwrap_or(".").to_string();
            vals[gt_idx] = first;
            return vals.join(":");
        }
    }
    if let Some(gp_idx) = keys.iter().position(|k| *k == "GP") {
        if let Some(gp) = vals.get(gp_idx) {
            let probs = gp
                .split(',')
                .filter_map(|x| x.parse::<f64>().ok())
                .collect::<Vec<_>>();
            if probs.len() >= 3 {
                let best_idx = probs
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(1);
                let hap = if best_idx == 0 { "0" } else { "1" };
                return hap.to_string();
            }
        }
    }
    if let Some(pl_idx) = keys.iter().position(|k| *k == "PL") {
        if let Some(pl) = vals.get(pl_idx) {
            let scores = pl
                .split(',')
                .filter_map(|x| x.parse::<f64>().ok())
                .collect::<Vec<_>>();
            if scores.len() >= 3 {
                let best_idx = scores
                    .iter()
                    .enumerate()
                    .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(1);
                let hap = if best_idx == 0 { "0" } else { "1" };
                return hap.to_string();
            }
        }
    }
    sample.to_string()
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
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
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

    let called_tbi = output_vcf.with_extension("vcf.gz.tbi");
    atomic_write_bytes(&called_tbi, b"index-placeholder\n")?;
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
    atomic_write_bytes(&out_vcf, out.as_bytes())?;
    write_call_outputs(out_dir, CallStageKind::Gl, input_vcf, &out_vcf, params)
}

/// # Errors
/// Returns an error if inputs do not satisfy diploid calling contracts.
pub fn run_call_diploid_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfCallParams,
) -> Result<CallStageOutputs> {
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
    atomic_write_bytes(&out_vcf, out.as_bytes())?;
    write_call_outputs(out_dir, CallStageKind::Diploid, input_vcf, &out_vcf, params)
}

/// # Errors
/// Returns an error if pseudo-haploid output cannot be produced.
pub fn run_call_pseudohaploid_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &VcfCallParams,
) -> Result<CallStageOutputs> {
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
    atomic_write_bytes(&out_vcf, out.as_bytes())?;
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
    let out = run_call_diploid_stage(input_vcf, out_dir, params)?;
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
    pub warnings_json: PathBuf,
    pub damage_genotype_manifest_json: PathBuf,
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

fn env_library_type() -> String {
    std::env::var("BIJUX_LIBRARY_TYPE")
        .ok()
        .map(|v| v.to_ascii_lowercase())
        .filter(|v| matches!(v.as_str(), "ssdna" | "dsdna"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn env_damage_mask_mode() -> String {
    std::env::var("BIJUX_VCF_DAMAGE_MASK_MODE")
        .ok()
        .map(|v| v.to_ascii_lowercase())
        .filter(|v| v == "remove" || v == "mark")
        .unwrap_or_else(|| "remove".to_string())
}

fn env_terminal_damage_threshold() -> f64 {
    std::env::var("BIJUX_VCF_DAMAGE_TERMINAL_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(0.50)
}

fn env_pmd_min_default(udg_regime: DamageUdgRegime) -> f64 {
    if let Some(parsed) = std::env::var("BIJUX_VCF_DAMAGE_PMD_MIN")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        return parsed;
    }
    match udg_regime {
        DamageUdgRegime::Udg => 0.0,
        DamageUdgRegime::NonUdg => 3.0,
        DamageUdgRegime::Unknown => 1.0,
    }
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
include!("call_and_damage_tail.rs");
