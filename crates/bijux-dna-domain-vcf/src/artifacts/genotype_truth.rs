//! Pure genotype, ploidy and likelihood truth over explicitly acquired VCF observations.
use super::{
    likelihood_value_present, parse_called_genotype, parse_format_value_from_sample,
    parse_gt_from_sample, usize_to_u32_saturating, TinyVcfDocument, VcfGenotypeTruthSummaryV1,
    VCF_GENOTYPE_TRUTH_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Paths and caller IDs are report metadata; this request does not acquire them.
pub(super) struct GenotypeTruthInput<'a> {
    pub(super) document: &'a TinyVcfDocument,
    pub(super) input_vcf: &'a Path,
    pub(super) stage_id: &'a str,
    pub(super) tool_id: &'a str,
    pub(super) per_sample_missingness: BTreeMap<String, f64>,
}

pub(super) fn summarize(input: GenotypeTruthInput<'_>) -> VcfGenotypeTruthSummaryV1 {
    let GenotypeTruthInput { document: doc, input_vcf, stage_id, tool_id, per_sample_missingness } =
        input;
    let mut observed_ploidy_widths = BTreeSet::<u32>::new();
    let mut called_calls = 0_u64;
    let mut missing_calls = 0_u64;
    let mut reference_only_calls = 0_u64;
    let mut mixed_allele_calls = 0_u64;
    let mut alternate_only_calls = 0_u64;
    let mut phased_calls = 0_u64;
    let mut unphased_calls = 0_u64;
    let mut likelihood_fields_present = BTreeSet::<String>::new();
    let mut sites_with_likelihood_values = BTreeMap::<String, u64>::new();
    let mut sites_missing_likelihood_values = BTreeMap::<String, u64>::new();

    for record in &doc.records {
        let Some(format) = &record.format else {
            continue;
        };
        for field_name in ["GL", "GP", "PL"] {
            if !format.split(':').any(|field| field == field_name) {
                continue;
            }
            likelihood_fields_present.insert(field_name.to_string());
            let has_value = record.samples.iter().any(|sample_payload| {
                parse_format_value_from_sample(format, sample_payload, field_name)
                    .is_some_and(likelihood_value_present)
            });
            if has_value {
                *sites_with_likelihood_values.entry(field_name.to_string()).or_insert(0) += 1;
            } else {
                *sites_missing_likelihood_values.entry(field_name.to_string()).or_insert(0) += 1;
            }
        }
        for sample_payload in &record.samples {
            let Some(gt) = parse_gt_from_sample(format, sample_payload) else {
                continue;
            };
            let Some((phased, alleles)) = parse_called_genotype(gt) else {
                missing_calls += 1;
                continue;
            };
            called_calls += 1;
            observed_ploidy_widths.insert(usize_to_u32_saturating(alleles.len()));
            if phased {
                phased_calls += 1;
            } else {
                unphased_calls += 1;
            }
            if alleles.iter().all(|allele| *allele == 0) {
                reference_only_calls += 1;
            } else if alleles.iter().all(|allele| *allele > 0) {
                alternate_only_calls += 1;
            } else {
                mixed_allele_calls += 1;
            }
        }
    }

    VcfGenotypeTruthSummaryV1 {
        schema_version: VCF_GENOTYPE_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        input_vcf: input_vcf.to_path_buf(),
        sample_count: usize_to_u32_saturating(doc.samples.len()),
        variant_count: doc.records.len() as u64,
        observed_ploidy_widths: observed_ploidy_widths.into_iter().collect(),
        called_calls,
        missing_calls,
        reference_only_calls,
        mixed_allele_calls,
        alternate_only_calls,
        phased_calls,
        unphased_calls,
        per_sample_missingness,
        likelihood_fields_present: likelihood_fields_present.into_iter().collect(),
        sites_with_likelihood_values,
        sites_missing_likelihood_values,
    }
}

#[cfg(test)]
#[path = "genotype_truth/contracts.rs"]
mod contracts;
