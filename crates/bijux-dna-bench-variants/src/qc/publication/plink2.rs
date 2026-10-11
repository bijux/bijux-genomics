//! qc publication plink2.
use crate::qc::model::AlleleFrequencyRow;
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::SampleGenotypeStats;

pub(crate) fn build_plink2_smiss_tsv(summary: &LocalVcfQcSummary) -> String {
    let mut lines = vec!["#IID\tMISSING_CT\tOBS_CT\tF_MISS".to_string()];
    for row in &summary.sample_missingness {
        lines.push(format!(
            "{}\t{}\t{}\t{}",
            row.sample_id, row.missing_genotype_count, row.total_genotype_count, row.missingness
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink2_vmiss_tsv(summary: &LocalVcfQcSummary) -> String {
    let mut lines = vec!["#CHROM\tPOS\tID\tREF\tALT\tMISSING_CT\tOBS_CT\tF_MISS".to_string()];
    for row in &summary.variant_missingness {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.contig,
            row.position,
            row.variant_id,
            row.reference,
            row.alternate,
            row.missing_sample_count,
            row.total_sample_count,
            row.missingness
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink2_afreq_tsv(rows: &[AlleleFrequencyRow]) -> String {
    let mut lines = vec!["#CHROM\tID\tREF\tALT\tALT_FREQS\tOBS_CT".to_string()];
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.contig,
            row.variant_id,
            row.reference,
            row.alternate,
            row.allele_frequency,
            row.observed_alleles / 2
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink2_het_tsv(rows: &[SampleGenotypeStats]) -> String {
    let mut lines = vec!["#IID\tO(HOM)\tE(HOM)\tN(NM)\tF".to_string()];
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t0.0",
            row.sample_id,
            row.homozygous_ref_call_count + row.homozygous_alt_call_count,
            row.nonmissing_genotype_count,
            row.nonmissing_genotype_count
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink2_hardy_tsv(
    summary: &LocalVcfQcSummary,
    rows: &[AlleleFrequencyRow],
) -> String {
    let mut lines = vec!["#CHROM\tPOS\tID\tREF\tALT\tA1_FREQ\tP".to_string()];
    let pvalue = summary.hwe_summary.pvalue_mean.unwrap_or(0.0);
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.contig,
            row.position,
            row.variant_id,
            row.reference,
            row.alternate,
            row.allele_frequency,
            pvalue
        ));
    }
    format!("{}\n", lines.join("\n"))
}
