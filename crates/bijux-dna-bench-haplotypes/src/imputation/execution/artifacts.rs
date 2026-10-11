//! Explicit stable artifact projection from native imputation outputs.
use crate::imputation::model::DEFAULT_OUTPUT_ACCEPT_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_INFO_HIST_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_LOGS_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_MAF_BINS_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_MANIFEST_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_OVERLAP_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_PANEL_MISMATCH_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_QC_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_QC_TSV_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_WARNINGS_NAME;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::ImputeStageOutputs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct ImputationArtifacts {
    pub(crate) output_vcf: PathBuf,
    pub(crate) output_tbi: PathBuf,
    pub(crate) imputation_qc: PathBuf,
    pub(crate) imputation_qc_tsv: PathBuf,
    pub(crate) imputation_manifest: PathBuf,
    pub(crate) overlap_stats: PathBuf,
    pub(crate) warnings: PathBuf,
    pub(crate) imputation_accept: PathBuf,
    pub(crate) panel_mismatch: PathBuf,
    pub(crate) maf_bins: PathBuf,
    pub(crate) info_hist: PathBuf,
    pub(crate) logs: PathBuf,
}

pub(crate) fn materialize_outputs(
    stage_outputs: &ImputeStageOutputs,
    output_root: &Path,
) -> Result<ImputationArtifacts> {
    let output_vcf_path = output_root.join(DEFAULT_OUTPUT_VCF_NAME);

    let output_tbi_path = PathBuf::from(format!("{}.tbi", output_vcf_path.display()));

    let imputation_qc_path = output_root.join(DEFAULT_OUTPUT_QC_NAME);

    let imputation_qc_tsv_path = output_root.join(DEFAULT_OUTPUT_QC_TSV_NAME);

    let imputation_manifest_path = output_root.join(DEFAULT_OUTPUT_MANIFEST_NAME);

    let overlap_stats_path = output_root.join(DEFAULT_OUTPUT_OVERLAP_NAME);

    let warnings_path = output_root.join(DEFAULT_OUTPUT_WARNINGS_NAME);

    let imputation_accept_path = output_root.join(DEFAULT_OUTPUT_ACCEPT_NAME);

    let panel_mismatch_path = output_root.join(DEFAULT_OUTPUT_PANEL_MISMATCH_NAME);

    let maf_bins_path = output_root.join(DEFAULT_OUTPUT_MAF_BINS_NAME);

    let info_hist_path = output_root.join(DEFAULT_OUTPUT_INFO_HIST_NAME);

    let logs_path = output_root.join(DEFAULT_OUTPUT_LOGS_NAME);

    for (source, destination) in [
        (&stage_outputs.imputed_vcf, &output_vcf_path),
        (&stage_outputs.imputed_tbi, &output_tbi_path),
        (&stage_outputs.imputation_qc_json, &imputation_qc_path),
        (&stage_outputs.imputation_qc_tsv, &imputation_qc_tsv_path),
        (&stage_outputs.imputation_manifest_json, &imputation_manifest_path),
        (&stage_outputs.overlap_stats_json, &overlap_stats_path),
        (&stage_outputs.warnings_json, &warnings_path),
        (&stage_outputs.imputation_accept_json, &imputation_accept_path),
        (&stage_outputs.panel_mismatch_diagnostics_json, &panel_mismatch_path),
        (&stage_outputs.maf_bin_quality_tsv, &maf_bins_path),
        (&stage_outputs.info_hist_json, &info_hist_path),
        (&stage_outputs.logs_txt, &logs_path),
    ] {
        bijux_dna_infra::copy_file(source, destination)?;
    }
    Ok(ImputationArtifacts {
        output_vcf: output_vcf_path,
        output_tbi: output_tbi_path,
        imputation_qc: imputation_qc_path,
        imputation_qc_tsv: imputation_qc_tsv_path,
        imputation_manifest: imputation_manifest_path,
        overlap_stats: overlap_stats_path,
        warnings: warnings_path,
        imputation_accept: imputation_accept_path,
        panel_mismatch: panel_mismatch_path,
        maf_bins: maf_bins_path,
        info_hist: info_hist_path,
        logs: logs_path,
    })
}
