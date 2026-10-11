//! Render explicit tool-report formats from internal QC observations; no external tool is executed.
use crate::qc::admission::report_format::QcReportFormat;
use crate::qc::model::AlleleFrequencyRow;
use crate::qc::model::GovernedQcOutputFile;
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::SampleGenotypeStats;
use crate::qc::observations::variants::summarize_qc_input_vcf;
use crate::qc::publication::plink::build_plink_frq_tsv;
use crate::qc::publication::plink::build_plink_het_tsv;
use crate::qc::publication::plink::build_plink_hwe_tsv;
use crate::qc::publication::plink::build_plink_imiss_tsv;
use crate::qc::publication::plink::build_plink_lmiss_tsv;
use crate::qc::publication::plink2::build_plink2_afreq_tsv;
use crate::qc::publication::plink2::build_plink2_hardy_tsv;
use crate::qc::publication::plink2::build_plink2_het_tsv;
use crate::qc::publication::plink2::build_plink2_smiss_tsv;
use crate::qc::publication::plink2::build_plink2_vmiss_tsv;
use crate::qc::publication::reports::build_bcftools_allele_frequency_tsv;
use crate::qc::publication::reports::build_bcftools_heterozygosity_tsv;
use crate::qc::publication::reports::build_hwe_summary_tsv;
use crate::qc::publication::reports::build_qc_log;
use crate::qc::publication::reports::build_sample_missingness_tsv;
use crate::qc::publication::reports::build_variant_missingness_tsv;
use anyhow::Result;
use std::path::Path;

pub(crate) fn materialize_governed_qc_contract_outputs(
    output_root: &Path,
    input_vcf: &Path,
    summary: &LocalVcfQcSummary,
    format: QcReportFormat,
) -> Result<Vec<GovernedQcOutputFile>> {
    let (allele_frequency_rows, sample_genotype_stats) = summarize_qc_input_vcf(input_vcf)?;
    match format {
        QcReportFormat::Bcftools => materialize_bcftools_qc_outputs(
            output_root,
            summary,
            &allele_frequency_rows,
            &sample_genotype_stats,
        ),
        QcReportFormat::Plink => materialize_plink_qc_outputs(
            output_root,
            summary,
            &allele_frequency_rows,
            &sample_genotype_stats,
        ),
        QcReportFormat::Plink2 => materialize_plink2_qc_outputs(
            output_root,
            summary,
            &allele_frequency_rows,
            &sample_genotype_stats,
        ),
    }
}

pub(crate) fn materialize_bcftools_qc_outputs(
    output_root: &Path,
    summary: &LocalVcfQcSummary,
    allele_frequency_rows: &[AlleleFrequencyRow],
    sample_genotype_stats: &[SampleGenotypeStats],
) -> Result<Vec<GovernedQcOutputFile>> {
    let sample_missingness_path = output_root.join("sample_missingness_report.tsv");
    let variant_missingness_path = output_root.join("variant_missingness_report.tsv");
    let allele_frequency_path = output_root.join("allele_frequency_report.tsv");
    let heterozygosity_path = output_root.join("heterozygosity_report.tsv");
    let hardy_weinberg_path = output_root.join("hardy_weinberg_report.tsv");
    let qc_thresholds_path = output_root.join("qc_thresholds.json");
    bijux_dna_infra::atomic_write_bytes(
        &sample_missingness_path,
        build_sample_missingness_tsv(summary).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &variant_missingness_path,
        build_variant_missingness_tsv(summary).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &allele_frequency_path,
        build_bcftools_allele_frequency_tsv(allele_frequency_rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &heterozygosity_path,
        build_bcftools_heterozygosity_tsv(sample_genotype_stats).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &hardy_weinberg_path,
        build_hwe_summary_tsv(summary).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_json(
        &qc_thresholds_path,
        &serde_json::json!({
            "sample_missingness_exclusion_threshold": summary.sample_missingness_exclusion_threshold,
            "variant_missingness_exclusion_threshold": summary.variant_missingness_exclusion_threshold,
            "excluded_samples": summary.excluded_samples,
            "excluded_variants": summary.excluded_variants,
        }),
    )?;
    Ok(vec![
        GovernedQcOutputFile {
            artifact_id: "sample_missingness_report",
            declared_path: "sample_missingness_report.tsv".to_string(),
            path: sample_missingness_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "variant_missingness_report",
            declared_path: "variant_missingness_report.tsv".to_string(),
            path: variant_missingness_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "allele_frequency_report",
            declared_path: "allele_frequency_report.tsv".to_string(),
            path: allele_frequency_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "heterozygosity_report",
            declared_path: "heterozygosity_report.tsv".to_string(),
            path: heterozygosity_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "hardy_weinberg_report",
            declared_path: "hardy_weinberg_report.tsv".to_string(),
            path: hardy_weinberg_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "qc_thresholds",
            declared_path: "qc_thresholds.json".to_string(),
            path: qc_thresholds_path,
            role: "report_output",
        },
    ])
}

pub(crate) fn materialize_plink_qc_outputs(
    output_root: &Path,
    summary: &LocalVcfQcSummary,
    allele_frequency_rows: &[AlleleFrequencyRow],
    sample_genotype_stats: &[SampleGenotypeStats],
) -> Result<Vec<GovernedQcOutputFile>> {
    let imiss_path = output_root.join("plink.imiss");
    let lmiss_path = output_root.join("plink.lmiss");
    let frq_path = output_root.join("plink.frq");
    let het_path = output_root.join("plink.het");
    let hwe_path = output_root.join("plink.hwe");
    let log_path = output_root.join("plink.log");
    bijux_dna_infra::atomic_write_bytes(&imiss_path, build_plink_imiss_tsv(summary).as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&lmiss_path, build_plink_lmiss_tsv(summary).as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(
        &frq_path,
        build_plink_frq_tsv(allele_frequency_rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &het_path,
        build_plink_het_tsv(sample_genotype_stats).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &hwe_path,
        build_plink_hwe_tsv(summary, allele_frequency_rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(&log_path, build_qc_log("plink", summary).as_bytes())?;
    Ok(vec![
        GovernedQcOutputFile {
            artifact_id: "sample_missingness_imiss",
            declared_path: "plink.imiss".to_string(),
            path: imiss_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "variant_missingness_lmiss",
            declared_path: "plink.lmiss".to_string(),
            path: lmiss_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "allele_frequency_frq",
            declared_path: "plink.frq".to_string(),
            path: frq_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "heterozygosity_het",
            declared_path: "plink.het".to_string(),
            path: het_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "hardy_weinberg_hwe",
            declared_path: "plink.hwe".to_string(),
            path: hwe_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "plink_log",
            declared_path: "plink.log".to_string(),
            path: log_path,
            role: "log_output",
        },
    ])
}

pub(crate) fn materialize_plink2_qc_outputs(
    output_root: &Path,
    summary: &LocalVcfQcSummary,
    allele_frequency_rows: &[AlleleFrequencyRow],
    sample_genotype_stats: &[SampleGenotypeStats],
) -> Result<Vec<GovernedQcOutputFile>> {
    let smiss_path = output_root.join("plink2.smiss");
    let vmiss_path = output_root.join("plink2.vmiss");
    let afreq_path = output_root.join("plink2.afreq");
    let het_path = output_root.join("plink2.het");
    let hardy_path = output_root.join("plink2.hardy");
    let log_path = output_root.join("plink2.log");
    bijux_dna_infra::atomic_write_bytes(&smiss_path, build_plink2_smiss_tsv(summary).as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&vmiss_path, build_plink2_vmiss_tsv(summary).as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(
        &afreq_path,
        build_plink2_afreq_tsv(allele_frequency_rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &het_path,
        build_plink2_het_tsv(sample_genotype_stats).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &hardy_path,
        build_plink2_hardy_tsv(summary, allele_frequency_rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(&log_path, build_qc_log("plink2", summary).as_bytes())?;
    Ok(vec![
        GovernedQcOutputFile {
            artifact_id: "sample_missingness_smiss",
            declared_path: "plink2.smiss".to_string(),
            path: smiss_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "variant_missingness_vmiss",
            declared_path: "plink2.vmiss".to_string(),
            path: vmiss_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "allele_frequency_afreq",
            declared_path: "plink2.afreq".to_string(),
            path: afreq_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "heterozygosity_het",
            declared_path: "plink2.het".to_string(),
            path: het_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "hardy_weinberg_hardy",
            declared_path: "plink2.hardy".to_string(),
            path: hardy_path,
            role: "report_output",
        },
        GovernedQcOutputFile {
            artifact_id: "plink2_log",
            declared_path: "plink2.log".to_string(),
            path: log_path,
            role: "log_output",
        },
    ])
}
