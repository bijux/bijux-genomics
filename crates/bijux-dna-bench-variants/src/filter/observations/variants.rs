//! filter observations variants.
use crate::filter::model::FilteredOutputSummary;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn parse_vcf_record_count(vcf_path: &Path) -> Result<u64> {
    let raw = read_vcf_text(vcf_path)?;
    let count =
        raw.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')).count();
    u64::try_from(count).map_err(|_| anyhow!("VCF record count overflowed u64"))
}

pub(crate) fn summarize_filtered_output(vcf_path: &Path) -> Result<FilteredOutputSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut pass_variants = 0_u64;
    let mut failed_variants = 0_u64;
    let mut filter_ids = BTreeSet::<String>::new();
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 8 {
            bail!("filter smoke row is missing FILTER field: {line}");
        }
        let filter_field = parts[6];
        if filter_field == "PASS" || filter_field == "." {
            pass_variants += 1;
            continue;
        }
        failed_variants += 1;
        for tag in filter_field.split(';') {
            if !tag.is_empty() && tag != "PASS" && tag != "." {
                filter_ids.insert(tag.to_string());
            }
        }
    }
    Ok(FilteredOutputSummary {
        pass_variants,
        failed_variants,
        filter_ids: filter_ids.into_iter().collect(),
    })
}

pub(crate) fn extract_threshold(
    report: &serde_json::Value,
    pointer: &str,
    name: &str,
) -> Result<f64> {
    report
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("filter explain is missing `{name}` threshold"))
}
