//! qc publication plink.
use crate::qc::model::AlleleFrequencyRow;
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::SampleGenotypeStats;

pub(crate) fn build_plink_imiss_tsv(summary: &LocalVcfQcSummary) -> String {
    let mut lines = vec!["FID\tIID\tMISS_PHENO\tN_MISS\tN_GENO\tF_MISS".to_string()];
    for row in &summary.sample_missingness {
        lines.push(format!(
            "{}\t{}\t0\t{}\t{}\t{}",
            row.sample_id,
            row.sample_id,
            row.missing_genotype_count,
            row.total_genotype_count,
            row.missingness
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink_lmiss_tsv(summary: &LocalVcfQcSummary) -> String {
    let mut lines = vec!["CHR\tSNP\tN_MISS\tN_GENO\tF_MISS".to_string()];
    for row in &summary.variant_missingness {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}",
            row.contig,
            row.variant_id,
            row.missing_sample_count,
            row.total_sample_count,
            row.missingness
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink_frq_tsv(rows: &[AlleleFrequencyRow]) -> String {
    let mut lines = vec!["CHR\tSNP\tA1\tA2\tMAF\tNCHROBS".to_string()];
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.contig,
            row.variant_id,
            row.alternate,
            row.reference,
            row.allele_frequency,
            row.observed_alleles
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink_het_tsv(rows: &[SampleGenotypeStats]) -> String {
    let mut lines = vec!["FID\tIID\tO(HOM)\tE(HOM)\tN(NM)\tF".to_string()];
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t0.0",
            row.sample_id,
            row.sample_id,
            row.homozygous_ref_call_count + row.homozygous_alt_call_count,
            row.nonmissing_genotype_count,
            row.nonmissing_genotype_count
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(crate) fn build_plink_hwe_tsv(
    summary: &LocalVcfQcSummary,
    rows: &[AlleleFrequencyRow],
) -> String {
    let mut lines = vec!["CHR\tSNP\tTEST\tA1\tA2\tGENO\tO(HET)\tE(HET)\tP".to_string()];
    let pvalue = summary.hwe_summary.pvalue_mean.unwrap_or(0.0);
    for row in rows {
        lines.push(format!(
            "{}\t{}\tALL\t{}\t{}\t0/1\t0.0\t0.0\t{}",
            row.contig, row.variant_id, row.alternate, row.reference, pvalue
        ));
    }
    format!("{}\n", lines.join("\n"))
}
