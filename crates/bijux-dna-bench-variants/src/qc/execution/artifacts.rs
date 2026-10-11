//! Project native output files into the execution report layout.
use crate::qc::model::DEFAULT_OUTPUT_HISTOGRAMS_NAME;
use crate::qc::model::DEFAULT_OUTPUT_IMPUTATION_QC_NAME;
use crate::qc::model::DEFAULT_OUTPUT_QC_TABLES_NAME;
use crate::qc::model::DEFAULT_OUTPUT_SUMMARY_NAME;
use crate::qc::model::DEFAULT_OUTPUT_WARNINGS_NAME;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::QcStageOutputs;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct StageArtifacts {
    pub(crate) qc_summary: PathBuf,
    pub(crate) qc_tables: PathBuf,
    pub(crate) imputation_qc: PathBuf,
    pub(crate) warnings: PathBuf,
    pub(crate) qc_histograms: PathBuf,
}

pub(crate) fn materialize_outputs(
    stage_outputs: &QcStageOutputs,
    output_root: &Path,
) -> Result<StageArtifacts> {
    let qc_summary_path = output_root.join(DEFAULT_OUTPUT_SUMMARY_NAME);
    fs::copy(&stage_outputs.qc_summary_json, &qc_summary_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.qc_summary_json.display(), qc_summary_path.display())
    })?;
    let qc_tables_path = output_root.join(DEFAULT_OUTPUT_QC_TABLES_NAME);
    fs::copy(&stage_outputs.qc_tables_tsv, &qc_tables_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.qc_tables_tsv.display(), qc_tables_path.display())
    })?;
    let imputation_qc_path = output_root.join(DEFAULT_OUTPUT_IMPUTATION_QC_NAME);
    fs::copy(&stage_outputs.imputation_qc_tsv, &imputation_qc_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.imputation_qc_tsv.display(),
            imputation_qc_path.display()
        )
    })?;
    let warnings_path = output_root.join(DEFAULT_OUTPUT_WARNINGS_NAME);
    fs::copy(&stage_outputs.warnings_json, &warnings_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.warnings_json.display(), warnings_path.display())
    })?;
    let qc_histograms_path = output_root.join(DEFAULT_OUTPUT_HISTOGRAMS_NAME);
    fs::copy(&stage_outputs.qc_histograms_json, &qc_histograms_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.qc_histograms_json.display(),
            qc_histograms_path.display()
        )
    })?;

    Ok(StageArtifacts {
        qc_summary: qc_summary_path,
        qc_tables: qc_tables_path,
        imputation_qc: imputation_qc_path,
        warnings: warnings_path,
        qc_histograms: qc_histograms_path,
    })
}
