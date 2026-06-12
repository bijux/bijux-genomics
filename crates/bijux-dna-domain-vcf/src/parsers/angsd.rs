use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::MultiGzDecoder;

use crate::taxonomy::VcfDomainStage;

const ANGSD_TOOL_ID: &str = "angsd";

const RAW_COMMAND_NAME: &str = "raw.command.json";
const RAW_CALL_GL_VCF_NAME: &str = "raw.gl_sites.vcf";
const RAW_PSEUDOHAPLOID_VCF_NAME: &str = "raw.pseudohaploid.vcf";
const RAW_DAMAGE_REPORT_NAME: &str = "raw.damage_report.txt";
const RAW_DAMAGE_AUDIT_NAME: &str = "raw.damage_bias_audit_report.json";
const RAW_GL_PROPAGATION_INPUT_NAME: &str = "raw.input.vcf";
const RAW_GL_PROPAGATION_OUTPUT_NAME: &str = "raw.gl_propagated.vcf";
const RAW_GL_PROPAGATION_REPORT_NAME: &str = "raw.gl_propagation_report.json";

#[derive(Debug, Clone)]
struct ParsedVcfRecord {
    format: Option<String>,
    samples: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct ParsedVcfDocument {
    sample_ids: Vec<String>,
    records: Vec<ParsedVcfRecord>,
}

#[derive(Debug, Clone)]
struct LikelihoodSummary {
    likelihood_field: String,
    sites_with_likelihoods: u64,
    samples_with_likelihoods: u64,
    missing_likelihoods: u64,
}

#[derive(Debug, Clone)]
struct LikelihoodFieldSetSummary {
    fields: BTreeSet<String>,
    site_count: u64,
}

/// Normalize the governed raw `angsd` artifact set for a retained VCF stage.
///
/// # Errors
/// Returns an error when required raw artifacts are missing or malformed.
pub fn parse_angsd_stage_metrics(
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    match stage {
        VcfDomainStage::CallGl => parse_call_gl_metrics(artifact_root),
        VcfDomainStage::CallPseudohaploid => parse_call_pseudohaploid_metrics(artifact_root),
        VcfDomainStage::DamageFilter => parse_damage_filter_metrics(artifact_root),
        VcfDomainStage::GlPropagation => parse_gl_propagation_metrics(artifact_root),
        other => bail!("unsupported angsd VCF parser stage `{}`", other.as_str()),
    }
}

fn parse_call_gl_metrics(root: &Path) -> Result<serde_json::Value> {
    let command = read_json(&root.join(RAW_COMMAND_NAME))?;
    validate_stage_command(&command, "vcf.call_gl", &["-GL", "-doGlf", "-doMajorMinor", "-doMaf"])?;

    let summary = summarize_likelihood_values(&root.join(RAW_CALL_GL_VCF_NAME))?;
    let sample_count = parse_vcf_document(&root.join(RAW_CALL_GL_VCF_NAME))?.sample_ids.len();
    let expected_field = json_string(&command, "/likelihood_field", "likelihood_field")?;
    if summary.likelihood_field != expected_field {
        bail!(
            "angsd call_gl likelihood field drifted: parser saw `{}`, command declared `{}`",
            summary.likelihood_field,
            expected_field
        );
    }
    let _likelihood_model = json_string(&command, "/likelihood_model", "likelihood_model")?;

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.call_gl.v1",
        "stage_id": "vcf.call_gl",
        "tool_id": ANGSD_TOOL_ID,
        "likelihood_field": summary.likelihood_field,
        "sites_with_likelihoods": summary.sites_with_likelihoods,
        "samples_with_likelihoods": summary.samples_with_likelihoods,
        "missing_likelihoods": summary.missing_likelihoods,
        "sample_count": sample_count,
    }))
}

fn parse_call_pseudohaploid_metrics(root: &Path) -> Result<serde_json::Value> {
    let command = read_json(&root.join(RAW_COMMAND_NAME))?;
    validate_stage_command(&command, "vcf.call_pseudohaploid", &["-doHaploCall", "-seed"])?;

    let doc = parse_vcf_document(&root.join(RAW_PSEUDOHAPLOID_VCF_NAME))?;
    let sampling_policy = json_string(&command, "/sampling_policy", "sampling_policy")?;
    let random_seed = json_u64(&command, "/random_seed", "random_seed")?;

    let mut called_sites = 0_u64;
    let mut missing_sites = 0_u64;
    for record in &doc.records {
        let format = record
            .format
            .as_deref()
            .ok_or_else(|| anyhow!("pseudohaploid raw VCF row is missing FORMAT"))?;
        for sample in &record.samples {
            let gt = extract_sample_field(format, sample, "GT")?;
            if gt == "." {
                missing_sites += 1;
                continue;
            }
            let alleles = split_genotype(gt)?;
            if alleles.len() != 1 {
                bail!("angsd pseudohaploid genotype is not haploid-compatible: `{gt}`");
            }
            called_sites += 1;
        }
    }
    let target_sites = u64::try_from(doc.records.len())
        .map_err(|_| anyhow!("pseudohaploid target site overflow"))?;

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.call_pseudohaploid.v1",
        "stage_id": "vcf.call_pseudohaploid",
        "tool_id": ANGSD_TOOL_ID,
        "target_sites": target_sites,
        "covered_sites": target_sites,
        "called_sites": called_sites,
        "missing_sites": missing_sites,
        "sampling_policy": sampling_policy,
        "random_seed": random_seed,
        "sample_count": doc.sample_ids.len(),
    }))
}

fn parse_damage_filter_metrics(root: &Path) -> Result<serde_json::Value> {
    let command = read_json(&root.join(RAW_COMMAND_NAME))?;
    validate_stage_command(&command, "vcf.damage_filter", &["-doDamage", "-pmd"])?;
    let audit = read_json(&root.join(RAW_DAMAGE_AUDIT_NAME))?;
    let counts = read_key_value_report(&root.join(RAW_DAMAGE_REPORT_NAME))?;

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.damage_filter.v1",
        "stage_id": "vcf.damage_filter",
        "tool_id": ANGSD_TOOL_ID,
        "input_variants": lookup_report_u64(&counts, "input_variants")?,
        "removed_variants": lookup_report_u64(&counts, "removed_variants")?,
        "retained_variants": lookup_report_u64(&counts, "retained_variants")?,
        "low_quality_filtered_variants": lookup_report_u64(&counts, "low_quality_filtered_variants")?,
        "damage_ratio_filtered_variants": lookup_report_u64(&counts, "damage_ratio_filtered_variants")?,
        "terminal_damage_filtered_variants": lookup_report_u64(&counts, "terminal_damage_filtered_variants")?,
        "damage_context_rule": format_damage_context_rule(&command)?,
        "terminal_context_count": json_u64(&audit, "/terminal_context_count", "terminal_context_count")?,
        "sample_count": lookup_report_u64(&counts, "sample_count")?,
    }))
}

fn parse_gl_propagation_metrics(root: &Path) -> Result<serde_json::Value> {
    let command = read_json(&root.join(RAW_COMMAND_NAME))?;
    validate_stage_command(&command, "vcf.gl_propagation", &["-vcf-gl", "-doPost", "-doVcf"])?;
    let report = read_json(&root.join(RAW_GL_PROPAGATION_REPORT_NAME))?;

    let input = summarize_likelihood_field_set(&root.join(RAW_GL_PROPAGATION_INPUT_NAME))?;
    let output = summarize_likelihood_field_set(&root.join(RAW_GL_PROPAGATION_OUTPUT_NAME))?;
    let lost_fields = input.fields.difference(&output.fields).cloned().collect::<Vec<_>>();
    let reported_lost_fields =
        json_string_array(&report, "/lost_fields", "lost_fields").unwrap_or_default();
    if lost_fields != reported_lost_fields {
        bail!(
            "angsd gl_propagation lost field drifted: parser saw {lost_fields:?}, report declared {reported_lost_fields:?}"
        );
    }
    let sample_count =
        parse_vcf_document(&root.join(RAW_GL_PROPAGATION_OUTPUT_NAME))?.sample_ids.len();

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.gl_propagation.v1",
        "stage_id": "vcf.gl_propagation",
        "tool_id": ANGSD_TOOL_ID,
        "input_likelihood_fields": input.fields.into_iter().collect::<Vec<_>>(),
        "output_likelihood_fields": output.fields.into_iter().collect::<Vec<_>>(),
        "lost_fields": lost_fields,
        "site_count_before": input.site_count,
        "site_count_after": output.site_count,
        "sample_count": sample_count,
    }))
}

fn validate_stage_command(
    command: &serde_json::Value,
    stage_id: &str,
    required_flags: &[&str],
) -> Result<()> {
    let tool_id = json_string(command, "/tool_id", "tool_id")?;
    if tool_id != ANGSD_TOOL_ID {
        bail!("angsd parser expected tool_id `{ANGSD_TOOL_ID}`, found `{tool_id}`");
    }
    let declared_stage_id = json_string(command, "/stage_id", "stage_id")?;
    if declared_stage_id != stage_id {
        bail!("angsd parser expected stage_id `{stage_id}`, found `{declared_stage_id}`");
    }
    let argv = json_string_array(command, "/argv", "argv")?;
    for required_flag in required_flags {
        if !argv.iter().any(|part| part == required_flag) {
            bail!("angsd parser command metadata for `{stage_id}` is missing `{required_flag}`");
        }
    }
    Ok(())
}

fn format_damage_context_rule(command: &serde_json::Value) -> Result<String> {
    let mode = json_string(command, "/masking_strategy/mode", "masking_strategy.mode")?;
    let max_damage_ratio =
        json_f64(command, "/thresholds/max_damage_ratio", "thresholds.max_damage_ratio")?;
    let terminal_threshold = json_f64(
        command,
        "/thresholds/terminal_damage_threshold",
        "thresholds.terminal_damage_threshold",
    )?;
    let pmd_min = json_f64(command, "/thresholds/pmd_min", "thresholds.pmd_min")?;
    Ok(format!(
        "{mode}_ct_ga_with_ratio_gt_{max_damage_ratio:.2}_or_terminal_signal_ge_{terminal_threshold:.2}_or_pmd_lt_{pmd_min:.1}"
    ))
}

fn lookup_report_u64(
    report: &std::collections::BTreeMap<String, String>,
    key: &str,
) -> Result<u64> {
    report
        .get(key)
        .ok_or_else(|| anyhow!("raw report is missing `{key}`"))?
        .parse::<u64>()
        .with_context(|| format!("raw report field `{key}` is not a valid u64"))
}

fn read_key_value_report(path: &Path) -> Result<std::collections::BTreeMap<String, String>> {
    let raw = read_text(path)?;
    let mut rows = std::collections::BTreeMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('\t') else {
            bail!("raw report row is not tab-separated: {line}");
        };
        rows.insert(key.to_string(), value.to_string());
    }
    Ok(rows)
}

fn parse_vcf_document(path: &Path) -> Result<ParsedVcfDocument> {
    let raw = read_text(path)?;
    let mut doc = ParsedVcfDocument::default();
    let mut saw_header = false;
    for (line_index, raw_line) in raw.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() || line.starts_with("##") {
            continue;
        }
        if let Some(header) = line.strip_prefix("#CHROM\t") {
            saw_header = true;
            let fields = header.split('\t').collect::<Vec<_>>();
            if fields.len() >= 9 {
                doc.sample_ids = fields[8..].iter().map(|value| (*value).to_string()).collect();
            }
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 8 {
            bail!(
                "malformed raw VCF record at line {}: expected at least 8 fields",
                line_index + 1
            );
        }
        let format =
            if fields.len() >= 9 && fields[8] != "." { Some(fields[8].to_string()) } else { None };
        let samples = if fields.len() >= 10 {
            fields[9..].iter().map(|value| (*value).to_string()).collect()
        } else {
            Vec::new()
        };
        doc.records.push(ParsedVcfRecord { format, samples });
    }
    if !saw_header {
        bail!("raw VCF is missing #CHROM header");
    }
    Ok(doc)
}

fn summarize_likelihood_values(path: &Path) -> Result<LikelihoodSummary> {
    let raw = read_text(path)?;
    let mut likelihood_field = None::<String>;
    let mut sites_with_likelihoods = 0_u64;
    let mut missing_likelihoods = 0_u64;
    let mut samples_with_likelihoods = BTreeSet::<usize>::new();

    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("angsd GL raw VCF row is missing FORMAT/sample fields: {line}");
        }
        let format_tokens = fields[8].split(':').collect::<Vec<_>>();
        let field_name = ["GL", "GP", "PL"]
            .into_iter()
            .find(|candidate| format_tokens.iter().any(|token| token == candidate))
            .ok_or_else(|| anyhow!("angsd GL raw VCF row is missing GL/GP/PL in FORMAT: {line}"))?;
        let field_index = format_tokens
            .iter()
            .position(|token| *token == field_name)
            .ok_or_else(|| anyhow!("angsd GL raw VCF row lost {field_name} in FORMAT: {line}"))?;

        if let Some(previous) = &likelihood_field {
            if previous != field_name {
                bail!(
                    "angsd GL likelihood field drifted across rows: `{previous}` vs `{field_name}`"
                );
            }
        } else {
            likelihood_field = Some(field_name.to_string());
        }

        let mut row_has_likelihood = false;
        for (sample_index, sample_field) in fields[9..].iter().enumerate() {
            let sample_value = sample_field.split(':').nth(field_index).ok_or_else(|| {
                anyhow!("angsd GL raw sample field is missing {field_name} value: {line}")
            })?;
            if likelihood_value_is_missing(sample_value) {
                missing_likelihoods += 1;
                continue;
            }
            row_has_likelihood = true;
            samples_with_likelihoods.insert(sample_index);
        }
        if row_has_likelihood {
            sites_with_likelihoods += 1;
        }
    }

    Ok(LikelihoodSummary {
        likelihood_field: likelihood_field
            .ok_or_else(|| anyhow!("angsd GL raw VCF did not contain any GL/GP/PL fields"))?,
        sites_with_likelihoods,
        samples_with_likelihoods: u64::try_from(samples_with_likelihoods.len())
            .map_err(|_| anyhow!("angsd GL sample-with-likelihood count overflowed u64"))?,
        missing_likelihoods,
    })
}

fn summarize_likelihood_field_set(path: &Path) -> Result<LikelihoodFieldSetSummary> {
    let raw = read_text(path)?;
    let mut fields = BTreeSet::<String>::new();
    let mut site_count = 0_u64;
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 10 {
            bail!("angsd gl_propagation raw VCF row is missing FORMAT/sample fields: {line}");
        }
        site_count += 1;
        for token in parts[8].split(':') {
            if ["GL", "GP", "PL"].contains(&token) {
                fields.insert(token.to_string());
            }
        }
    }
    Ok(LikelihoodFieldSetSummary { fields, site_count })
}

fn extract_sample_field<'a>(format: &'a str, sample: &'a str, field_name: &str) -> Result<&'a str> {
    let keys = format.split(':').collect::<Vec<_>>();
    let index = keys
        .iter()
        .position(|token| *token == field_name)
        .ok_or_else(|| anyhow!("FORMAT field is missing `{field_name}`"))?;
    sample
        .split(':')
        .nth(index)
        .ok_or_else(|| anyhow!("sample payload is missing `{field_name}` value"))
}

fn split_genotype(genotype: &str) -> Result<Vec<u32>> {
    genotype
        .split(['/', '|'])
        .map(|allele| {
            allele
                .parse::<u32>()
                .with_context(|| format!("genotype allele is not numeric: `{genotype}`"))
        })
        .collect()
}

fn likelihood_value_is_missing(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.split(',').all(|token| matches!(token.trim(), "." | ""))
}

fn read_text(path: &Path) -> Result<String> {
    let mut buffer = String::new();
    if path.extension().and_then(|value| value.to_str()) == Some("gz") {
        let file = fs::File::open(path).with_context(|| format!("read {}", path.display()))?;
        let mut decoder = MultiGzDecoder::new(file);
        decoder
            .read_to_string(&mut buffer)
            .with_context(|| format!("decode {}", path.display()))?;
        return Ok(buffer);
    }
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn json_string(value: &serde_json::Value, pointer: &str, name: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))
}

fn json_u64(value: &serde_json::Value, pointer: &str, name: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))
}

fn json_f64(value: &serde_json::Value, pointer: &str, name: &str) -> Result<f64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))
}

fn json_string_array(value: &serde_json::Value, pointer: &str, name: &str) -> Result<Vec<String>> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("raw artifact `{name}` contains a non-string entry"))
        })
        .collect()
}
