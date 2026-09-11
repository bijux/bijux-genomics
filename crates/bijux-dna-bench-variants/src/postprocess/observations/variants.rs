//! postprocess observations variants.
use crate::postprocess::model::PostprocessOutputSummary;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use serde_json::Value;
use std::path::Path;

pub(crate) fn summarize_postprocess_output(vcf_path: &Path) -> Result<PostprocessOutputSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("postprocess output is missing the #CHROM header"))?;
    let sample_ids = sample_header.split('\t').skip(9).map(ToString::to_string).collect::<Vec<_>>();
    let record_lines = raw.lines().filter(|line| !line.starts_with('#')).collect::<Vec<_>>();
    let mq_removed = record_lines.iter().all(|line| {
        line.split('\t').nth(7).is_some_and(|info| {
            !info.split(';').any(|token| {
                token == "MQ=50" || token == "MQ=45" || token == "MQ=40" || token.starts_with("MQ=")
            })
        })
    });
    let multiallelic_records_remaining = record_lines
        .iter()
        .filter(|line| line.split('\t').nth(4).is_some_and(|alt| alt.contains(',')))
        .count();
    Ok(PostprocessOutputSummary {
        record_count: u64::try_from(record_lines.len())
            .map_err(|_| anyhow!("postprocess output record count overflowed u64"))?,
        sample_count: parse_output_sample_count(vcf_path)?,
        sample_ids,
        mq_removed,
        multiallelic_records_remaining: u64::try_from(multiallelic_records_remaining)
            .map_err(|_| anyhow!("multiallelic record count overflowed u64"))?,
    })
}

pub(crate) fn json_bool(value: &Value, pointer: &str) -> Result<bool> {
    value
        .pointer(pointer)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("missing boolean `{pointer}`"))
}

pub(crate) fn json_u64(value: &Value, pointer: &str) -> Result<u64> {
    value.pointer(pointer).and_then(Value::as_u64).ok_or_else(|| anyhow!("missing u64 `{pointer}`"))
}
