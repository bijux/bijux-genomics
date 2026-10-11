//! qc admission bindings.
use crate::inputs::VariantBenchmarkInputs;
use crate::qc::model::GovernedVcfQcSmokeContract;
use crate::qc::model::GOVERNED_VCF_QC_ASSET_PROFILE_ID;
use crate::qc::model::GOVERNED_VCF_QC_CORPUS_ID;
use crate::qc::model::GOVERNED_VCF_QC_INPUT_FIXTURE_ID;
use crate::qc::model::GOVERNED_VCF_QC_SAMPLE_NAME;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_qc_smoke_contract(
    inputs: &VariantBenchmarkInputs,
    requested_tool_id: &str,
) -> Result<GovernedVcfQcSmokeContract> {
    let matrix_row =
        inputs.qc().find(|&row| row.tool_id == requested_tool_id).cloned().ok_or_else(|| {
            anyhow!("VCF QC smoke is missing governed retained tool `{requested_tool_id}`")
        })?;
    if matrix_row.corpus_id != GOVERNED_VCF_QC_CORPUS_ID {
        bail!(
            "VCF QC smoke requires corpus `{GOVERNED_VCF_QC_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_QC_ASSET_PROFILE_ID {
        bail!(
            "VCF QC smoke requires asset profile `{GOVERNED_VCF_QC_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != ["qc_report"] {
        bail!("VCF QC smoke expected output must be qc_report");
    }
    let report_format = super::report_format::QcReportFormat::from_tool(&matrix_row.tool_id)?;
    Ok(GovernedVcfQcSmokeContract {
        report_format,
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_QC_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_QC_SAMPLE_NAME.to_string(),
    })
}
