use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::taxonomy::VcfDomainStage;

const RAW_COMMAND_NAME: &str = "raw.command.json";
const RAW_PHASED_VCF_NAME: &str = "raw.phased.vcf";
const RAW_PHASING_QC_NAME: &str = "raw.phasing_qc.json";
const RAW_PHASING_MANIFEST_NAME: &str = "raw.phasing_manifest.json";
const RAW_PHASE_BLOCK_STATS_NAME: &str = "raw.phase_block_stats.tsv";
const RAW_SWITCH_ERROR_PROXY_NAME: &str = "raw.switch_error_proxy.tsv";

#[derive(Debug, Clone)]
struct PhasingSummary {
    sample_ids: Vec<String>,
    input_genotypes: u64,
    phased_genotypes: u64,
    unphased_genotypes: u64,
    phase_set_count: u64,
    output_variant_count: u64,
}

/// Normalize the governed raw phasing artifact set for a retained VCF backend.
///
/// # Errors
/// Returns an error when required raw artifacts are missing, malformed, or unphased.
pub fn parse_phasing_stage_metrics(
    tool_id: &str,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    let stage = VcfDomainStage::Phasing;
    if stage.as_str() != "vcf.phasing" {
        bail!("phasing parser drifted away from vcf.phasing");
    }
    validate_command(tool_id, &read_json(&artifact_root.join(RAW_COMMAND_NAME))?)?;

    let phasing_qc = read_json(&artifact_root.join(RAW_PHASING_QC_NAME))?;
    let phasing_manifest = read_json(&artifact_root.join(RAW_PHASING_MANIFEST_NAME))?;
    validate_backend_field(tool_id, &phasing_qc, "phasing qc")?;
    validate_backend_field(tool_id, &phasing_manifest, "phasing manifest")?;
    validate_manifest(tool_id, &phasing_manifest)?;

    let phase_block_n50 =
        read_metric_table_u64(&artifact_root.join(RAW_PHASE_BLOCK_STATS_NAME), "phase_block_n50")?;
    let switch_error_proxy =
        read_metric_table(&artifact_root.join(RAW_SWITCH_ERROR_PROXY_NAME), "switch_error_proxy")?;
    let qc_phase_block_n50 = json_u64(&phasing_qc, "/phase_block_n50", "phase_block_n50")?;
    if phase_block_n50 != qc_phase_block_n50 {
        bail!(
            "phasing qc phase_block_n50 drifted: table={phase_block_n50}, qc={qc_phase_block_n50}"
        );
    }
    let qc_switch_error_proxy = json_f64(&phasing_qc, "/switch_error_proxy", "switch_error_proxy")?;
    if (switch_error_proxy - qc_switch_error_proxy).abs() > 1e-9 {
        bail!(
            "phasing qc switch_error_proxy drifted: table={switch_error_proxy}, qc={qc_switch_error_proxy}"
        );
    }

    let summary = summarize_phased_vcf(&artifact_root.join(RAW_PHASED_VCF_NAME))?;
    if summary.phased_genotypes == 0 && summary.unphased_genotypes > 0 {
        bail!("phasing output contains no phased genotypes");
    }
    if summary.phase_set_count == 0 && summary.phased_genotypes > 0 {
        bail!("phasing output is missing phase-set evidence");
    }

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.phasing.v1",
        "stage_id": "vcf.phasing",
        "tool_id": tool_id,
        "input_genotypes": summary.input_genotypes,
        "phased_genotypes": summary.phased_genotypes,
        "unphased_genotypes": summary.unphased_genotypes,
        "phase_set_count": summary.phase_set_count,
        "sample_count": summary.sample_ids.len(),
        "sample_ids": summary.sample_ids,
        "output_variant_count": summary.output_variant_count,
        "status": "complete",
    }))
}

fn validate_command(tool_id: &str, command: &serde_json::Value) -> Result<()> {
    let declared_tool_id = json_string(command, "/tool_id", "tool_id")?;
    if declared_tool_id != tool_id {
        bail!("phasing parser expected tool_id `{tool_id}`, found `{declared_tool_id}`");
    }
    let declared_stage_id = json_string(command, "/stage_id", "stage_id")?;
    if declared_stage_id != "vcf.phasing" {
        bail!("phasing parser expected stage_id `vcf.phasing`, found `{declared_stage_id}`");
    }
    let argv = json_string_array(command, "/argv", "argv")?;
    for token in required_command_tokens(tool_id)? {
        if !argv.iter().any(|part| part.contains(token)) {
            bail!("phasing command for `{tool_id}` is missing `{token}`");
        }
    }
    Ok(())
}

fn validate_backend_field(tool_id: &str, value: &serde_json::Value, surface: &str) -> Result<()> {
    let backend = json_string(value, "/backend", "backend")?;
    if backend != tool_id {
        bail!("{surface} drifted away from backend `{tool_id}`: found `{backend}`");
    }
    Ok(())
}

fn validate_manifest(tool_id: &str, manifest: &serde_json::Value) -> Result<()> {
    let stage_id = json_string(manifest, "/stage_id", "stage_id")?;
    if stage_id != "vcf.phasing" {
        bail!("phasing manifest stage drifted: found `{stage_id}`");
    }
    let argv = json_string_array(manifest, "/command_argv", "command_argv")?;
    for token in required_command_tokens(tool_id)? {
        if !argv.iter().any(|part| part.contains(token)) {
            bail!("phasing manifest for `{tool_id}` is missing `{token}`");
        }
    }
    Ok(())
}

fn required_command_tokens(tool_id: &str) -> Result<&'static [&'static str]> {
    match tool_id {
        "shapeit5" => Ok(&["shapeit5", "phase_common", "--reference", "--map", "--output"]),
        "eagle" => Ok(&["eagle", "--vcfTarget", "--vcfRef", "--geneticMapFile", "--outPrefix"]),
        "beagle" => Ok(&["beagle", "gt=", "ref=", "map=", "out="]),
        _ => bail!("unsupported phasing tool `{tool_id}`"),
    }
}

fn summarize_phased_vcf(path: &Path) -> Result<PhasingSummary> {
    let raw = read_text(path)?;
    let header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("phased VCF is missing the #CHROM header"))?;
    let sample_ids = header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
    if sample_ids.is_empty() {
        bail!("phased VCF does not declare any samples");
    }

    let mut phase_sets_by_sample = vec![BTreeMap::<String, ()>::new(); sample_ids.len()];
    let mut input_genotypes = 0_u64;
    let mut phased_genotypes = 0_u64;
    let mut unphased_genotypes = 0_u64;
    let mut output_variant_count = 0_u64;

    for line in raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()) {
        output_variant_count += 1;
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("phased VCF row is missing sample columns");
        }
        let format_keys = fields[8].split(':').collect::<Vec<_>>();
        let ps_index = format_keys.iter().position(|field| *field == "PS");
        for (sample_index, sample_field) in fields.iter().skip(9).enumerate() {
            let values = sample_field.split(':').collect::<Vec<_>>();
            let gt = values.first().copied().unwrap_or_default();
            if gt.is_empty() || gt == "." || gt == "./." || gt == ".|." {
                continue;
            }
            input_genotypes += 1;
            if gt.contains('|') {
                phased_genotypes += 1;
                if let Some(ps_index) = ps_index {
                    if let Some(ps_value) = values.get(ps_index) {
                        if !ps_value.is_empty() && *ps_value != "." {
                            phase_sets_by_sample[sample_index].insert((*ps_value).to_string(), ());
                        }
                    }
                }
            } else if gt.contains('/') {
                unphased_genotypes += 1;
            }
        }
    }

    let phase_set_count = if phase_sets_by_sample.iter().any(|row| !row.is_empty()) {
        phase_sets_by_sample.iter().map(|row| u64::try_from(row.len()).unwrap_or(0)).sum()
    } else if phased_genotypes > 0 {
        u64::try_from(sample_ids.len()).unwrap_or(0)
    } else {
        0
    };

    Ok(PhasingSummary {
        sample_ids,
        input_genotypes,
        phased_genotypes,
        unphased_genotypes,
        phase_set_count,
        output_variant_count,
    })
}

fn read_metric_table(path: &Path, expected_key: &str) -> Result<f64> {
    let rows = read_rows(path)?;
    let (_header, data_rows) =
        rows.split_first().ok_or_else(|| anyhow!("metric table {} is empty", path.display()))?;
    let row = data_rows
        .iter()
        .find(|row| row.first().map(String::as_str) == Some(expected_key))
        .ok_or_else(|| anyhow!("metric table {} is missing `{expected_key}`", path.display()))?;
    let value = row.get(1).ok_or_else(|| {
        anyhow!("metric table {} row `{expected_key}` is missing value", path.display())
    })?;
    value.parse::<f64>().with_context(|| format!("parse `{expected_key}` in {}", path.display()))
}

fn read_metric_table_u64(path: &Path, expected_key: &str) -> Result<u64> {
    let rows = read_rows(path)?;
    let (_header, data_rows) =
        rows.split_first().ok_or_else(|| anyhow!("metric table {} is empty", path.display()))?;
    let row = data_rows
        .iter()
        .find(|row| row.first().map(String::as_str) == Some(expected_key))
        .ok_or_else(|| anyhow!("metric table {} is missing `{expected_key}`", path.display()))?;
    let value = row.get(1).ok_or_else(|| {
        anyhow!("metric table {} row `{expected_key}` is missing value", path.display())
    })?;
    value.parse::<u64>().with_context(|| format!("parse `{expected_key}` in {}", path.display()))
}

fn read_rows(path: &Path) -> Result<Vec<Vec<String>>> {
    let raw = read_text(path)?;
    Ok(raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split_whitespace().map(str::to_string).collect())
        .collect())
}

fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = read_text(path)?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn json_string(value: &serde_json::Value, pointer: &str, field: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not a string"))
}

fn json_string_array(value: &serde_json::Value, pointer: &str, field: &str) -> Result<Vec<String>> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not an array"))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("json field `{field}` contains a non-string entry"))
        })
        .collect()
}

fn json_u64(value: &serde_json::Value, pointer: &str, field: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not a u64"))
}

fn json_f64(value: &serde_json::Value, pointer: &str, field: &str) -> Result<f64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not an f64"))
}
