//! imputation observations variants.
use crate::imputation::model::ImputeOutputSummary;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::path::Path;

pub(crate) fn summarize_imputed_output(vcf_path: &Path) -> Result<ImputeOutputSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("imputed VCF is missing the #CHROM header"))?;
    let sample_ids = sample_header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
    let first_record = raw
        .lines()
        .find(|line| !line.starts_with('#') && !line.trim().is_empty())
        .ok_or_else(|| anyhow!("imputed VCF is missing records"))?;
    let fields = first_record.split('\t').collect::<Vec<_>>();
    let gt_idx = format_index(&fields, "GT").ok_or_else(|| anyhow!("imputed VCF missing GT"))?;
    let masked_sample_gt = fields
        .get(9)
        .and_then(|sample| sample.split(':').collect::<Vec<_>>().get(gt_idx).copied())
        .ok_or_else(|| anyhow!("imputed VCF missing masked sample GT"))?
        .to_string();
    let donor_sample_gt = fields
        .get(10)
        .and_then(|sample| sample.split(':').collect::<Vec<_>>().get(gt_idx).copied())
        .ok_or_else(|| anyhow!("imputed VCF missing donor sample GT"))?
        .to_string();
    let variant_count =
        raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()).count() as u64;
    Ok(ImputeOutputSummary { variant_count, sample_ids, masked_sample_gt, donor_sample_gt })
}

pub(crate) fn format_index(fields: &[&str], name: &str) -> Option<usize> {
    fields.get(8)?.split(':').position(|field| field == name)
}
