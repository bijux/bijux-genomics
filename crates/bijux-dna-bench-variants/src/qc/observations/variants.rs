//! Observe cohort genotypes and compose scientific admission.
use crate::qc::model::AlleleFrequencyRow;
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::SampleGenotypeStats;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn validate_qc_summary(summary: &LocalVcfQcSummary) -> Result<()> {
    super::cohort_metrics::validate(summary)?;
    super::sample_exclusions::validate(summary)?;
    super::variant_exclusions::validate(summary)
}

pub(crate) fn summarize_qc_input_vcf(
    input_vcf: &Path,
) -> Result<(Vec<AlleleFrequencyRow>, Vec<SampleGenotypeStats>)> {
    let raw =
        fs::read_to_string(input_vcf).with_context(|| format!("read {}", input_vcf.display()))?;
    let mut sample_ids = Vec::<String>::new();
    let mut sample_stats = BTreeMap::<String, SampleGenotypeStats>::new();
    let mut allele_rows = Vec::<AlleleFrequencyRow>::new();
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            sample_ids = line.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
            for sample_id in &sample_ids {
                sample_stats.insert(
                    sample_id.clone(),
                    SampleGenotypeStats {
                        sample_id: sample_id.clone(),
                        nonmissing_genotype_count: 0,
                        heterozygous_call_count: 0,
                        homozygous_alt_call_count: 0,
                        homozygous_ref_call_count: 0,
                    },
                );
            }
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            continue;
        }
        let mut alt_alleles = 0_u64;
        let mut observed_alleles = 0_u64;
        for (index, sample_field) in fields[9..].iter().enumerate() {
            let gt = sample_field.split(':').next().unwrap_or_default().replace('|', "/");
            let Some(sample_id) = sample_ids.get(index) else {
                continue;
            };
            let Some(stats) = sample_stats.get_mut(sample_id) else {
                continue;
            };
            if gt.contains('.') {
                continue;
            }
            stats.nonmissing_genotype_count += 1;
            match gt.as_str() {
                "0/0" => stats.homozygous_ref_call_count += 1,
                "0/1" | "1/0" => stats.heterozygous_call_count += 1,
                "1/1" => stats.homozygous_alt_call_count += 1,
                _ => {}
            }
            for allele in gt.split('/') {
                if allele == "." {
                    continue;
                }
                observed_alleles += 1;
                if allele == "1" {
                    alt_alleles += 1;
                }
            }
        }
        // Observed integer ratios are represented as floating-point scientific metrics.
        #[allow(clippy::cast_precision_loss)]
        let allele_frequency =
            if observed_alleles == 0 { 0.0 } else { alt_alleles as f64 / observed_alleles as f64 };
        allele_rows.push(AlleleFrequencyRow {
            variant_id: format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]),
            contig: fields[0].to_string(),
            position: fields[1].parse::<u64>().unwrap_or(0),
            reference: fields[3].to_string(),
            alternate: fields[4].to_string(),
            allele_frequency,
            observed_alleles,
        });
    }
    Ok((allele_rows, sample_stats.into_values().collect::<Vec<_>>()))
}
