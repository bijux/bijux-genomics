//! Capture imputation outputs for downstream quality reporting.
use crate::imputation::model::LocalVcfImputeSmokeReport;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_ORCHESTRATION_MANIFEST_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_ACCEPT_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_QC_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_SMOKE_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_TBI_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_VCF_NAME;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct SourceArtifacts {
    pub(crate) source_imputed_vcf: PathBuf,
    pub(crate) source_imputed_tbi: PathBuf,
    pub(crate) source_imputation_accept: PathBuf,
    pub(crate) source_imputation_qc: PathBuf,
    pub(crate) source_impute_smoke_metrics: PathBuf,
    pub(crate) source_imputation_manifest: PathBuf,
    pub(crate) source_logs: PathBuf,
    pub(crate) orchestration_manifest: PathBuf,
}

pub(crate) fn materialize_source(
    repo_root: &Path,
    output_root: &Path,
    source_report: &LocalVcfImputeSmokeReport,
) -> Result<SourceArtifacts> {
    let source_imputation_qc_source = repo_root.join(&source_report.imputation_qc_path);
    let source_imputed_vcf_source = repo_root.join(&source_report.output_vcf_path);
    let source_imputed_tbi_source = repo_root.join(&source_report.output_tbi_path);
    let source_imputation_accept_source = repo_root.join(&source_report.imputation_accept_path);
    let source_impute_smoke_metrics_source = repo_root.join(&source_report.metrics_path);
    let source_imputation_manifest_source = repo_root.join(&source_report.imputation_manifest_path);
    let source_logs_source = repo_root.join(&source_report.logs_path);
    let source_imputed_vcf_path = output_root.join(DEFAULT_OUTPUT_SOURCE_VCF_NAME);
    let source_imputed_tbi_path = output_root.join(DEFAULT_OUTPUT_SOURCE_TBI_NAME);
    let source_imputation_accept_path = output_root.join(DEFAULT_OUTPUT_SOURCE_ACCEPT_NAME);
    let source_imputation_qc_path = output_root.join(DEFAULT_OUTPUT_SOURCE_QC_NAME);
    let source_impute_smoke_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SMOKE_NAME);
    let source_imputation_manifest_path = output_root.join(DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME);
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    let orchestration_manifest_path = output_root.join(DEFAULT_OUTPUT_ORCHESTRATION_MANIFEST_NAME);

    bijux_dna_infra::copy_file(&source_imputed_vcf_source, &source_imputed_vcf_path).with_context(
        || {
            format!(
                "copy {} to {}",
                source_imputed_vcf_source.display(),
                source_imputed_vcf_path.display()
            )
        },
    )?;
    bijux_dna_infra::copy_file(&source_imputed_tbi_source, &source_imputed_tbi_path).with_context(
        || {
            format!(
                "copy {} to {}",
                source_imputed_tbi_source.display(),
                source_imputed_tbi_path.display()
            )
        },
    )?;
    bijux_dna_infra::copy_file(&source_imputation_accept_source, &source_imputation_accept_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                source_imputation_accept_source.display(),
                source_imputation_accept_path.display()
            )
        })?;
    bijux_dna_infra::copy_file(&source_imputation_qc_source, &source_imputation_qc_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                source_imputation_qc_source.display(),
                source_imputation_qc_path.display()
            )
        })?;
    bijux_dna_infra::copy_file(
        &source_impute_smoke_metrics_source,
        &source_impute_smoke_metrics_path,
    )
    .with_context(|| {
        format!(
            "copy {} to {}",
            source_impute_smoke_metrics_source.display(),
            source_impute_smoke_metrics_path.display()
        )
    })?;
    bijux_dna_infra::copy_file(
        &source_imputation_manifest_source,
        &source_imputation_manifest_path,
    )
    .with_context(|| {
        format!(
            "copy {} to {}",
            source_imputation_manifest_source.display(),
            source_imputation_manifest_path.display()
        )
    })?;
    bijux_dna_infra::copy_file(&source_logs_source, &source_logs_path).with_context(|| {
        format!("copy {} to {}", source_logs_source.display(), source_logs_path.display())
    })?;

    Ok(SourceArtifacts {
        source_imputed_vcf: source_imputed_vcf_path,
        source_imputed_tbi: source_imputed_tbi_path,
        source_imputation_accept: source_imputation_accept_path,
        source_imputation_qc: source_imputation_qc_path,
        source_impute_smoke_metrics: source_impute_smoke_metrics_path,
        source_imputation_manifest: source_imputation_manifest_path,
        source_logs: source_logs_path,
        orchestration_manifest: orchestration_manifest_path,
    })
}
