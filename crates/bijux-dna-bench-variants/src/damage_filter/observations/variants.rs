//! damage filter observations variants.
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::path::Path;

pub(crate) fn parse_vcf_record_count(vcf_path: &Path) -> Result<u64> {
    let raw = read_vcf_text(vcf_path)?;
    let count =
        raw.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')).count();
    u64::try_from(count).map_err(|_| anyhow!("VCF record count overflowed u64"))
}

pub(crate) fn format_damage_context_rule(summary_json: &serde_json::Value) -> Result<String> {
    let mode = summary_json
        .pointer("/masking_strategy/mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("damage_filter summary is missing masking mode"))?;
    let max_damage_ratio = summary_json
        .pointer("/thresholds/max_damage_ratio")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("damage_filter summary is missing max_damage_ratio"))?;
    let terminal_damage_threshold = summary_json
        .pointer("/thresholds/terminal_damage_threshold")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("damage_filter summary is missing terminal_damage_threshold"))?;
    let pmd_min = summary_json
        .pointer("/thresholds/pmd_min")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("damage_filter summary is missing pmd_min"))?;
    Ok(format!(
        "{mode}_ct_ga_with_ratio_gt_{max_damage_ratio:.2}_or_terminal_signal_ge_{terminal_damage_threshold:.2}_or_pmd_lt_{pmd_min:.1}"
    ))
}
