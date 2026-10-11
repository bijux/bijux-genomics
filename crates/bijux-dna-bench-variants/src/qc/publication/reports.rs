//! qc publication reports.
use crate::qc::model::AlleleFrequencyRow;
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::SampleGenotypeStats;

pub(crate) fn build_sample_missingness_tsv(summary: &LocalVcfQcSummary) -> String {
    let mut lines =
        vec!["sample_id\ttotal_genotype_count\tmissing_genotype_count\tmissingness".to_string()];
    for row in &summary.sample_missingness {
        lines.push(format!(
            "{}\t{}\t{}\t{}",
            row.sample_id, row.total_genotype_count, row.missing_genotype_count, row.missingness
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_variant_missingness_tsv(summary: &LocalVcfQcSummary) -> String {
    let mut lines = vec![
        "variant_id\tcontig\tposition\treference\talternate\ttotal_sample_count\tmissing_sample_count\tmissingness"
            .to_string(),
    ];
    for row in &summary.variant_missingness {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.variant_id,
            row.contig,
            row.position,
            row.reference,
            row.alternate,
            row.total_sample_count,
            row.missing_sample_count,
            row.missingness
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_bcftools_allele_frequency_tsv(rows: &[AlleleFrequencyRow]) -> String {
    let mut lines = vec!["variant_id\tallele_frequency\tobserved_alleles".to_string()];
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}",
            row.variant_id, row.allele_frequency, row.observed_alleles
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_bcftools_heterozygosity_tsv(rows: &[SampleGenotypeStats]) -> String {
    let mut lines = vec![
        "sample_id\tnonmissing_genotype_count\theterozygous_call_count\thomozygous_alt_call_count\thomozygous_ref_call_count".to_string(),
    ];
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}",
            row.sample_id,
            row.nonmissing_genotype_count,
            row.heterozygous_call_count,
            row.homozygous_alt_call_count,
            row.homozygous_ref_call_count
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_hwe_summary_tsv(summary: &LocalVcfQcSummary) -> String {
    format!(
        "tested_variant_count\tpvalue_mean\tstatus\n{}\t{}\t{}\n",
        summary.hwe_summary.tested_variant_count,
        summary.hwe_summary.pvalue_mean.unwrap_or(0.0),
        summary.hwe_summary.status
    )
}

pub(crate) fn build_qc_log(tool_id: &str, summary: &LocalVcfQcSummary) -> String {
    format!(
        "{tool_id} qc smoke\nexecutor=bijux_internal_vcf_qc\nreport_format={tool_id}\nexternal_tool_executed=false\nsample_threshold={}\nvariant_threshold={}\nexcluded_samples={}\nexcluded_variants={}\n",
        summary.sample_missingness_exclusion_threshold,
        summary.variant_missingness_exclusion_threshold,
        summary.excluded_samples.len(),
        summary.excluded_variants.len()
    )
}
