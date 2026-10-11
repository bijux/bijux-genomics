//! call bam support admission bindings.
use crate::bam::model::GovernedVcfCallBamSmokeContract;
use crate::bam::model::GOVERNED_BAM_SAMPLE_ID;
use crate::bam::model::GOVERNED_VCF_CALL_TOOL_ID;
use crate::inputs::GenotypeBenchmarkInputs;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use bijux_dna_bench_corpus::bam::validate_bam_corpus_fixture_manifest_path;
use bijux_dna_bench_corpus::bam::DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH;
use bijux_dna_bench_model::VcfStageMatrixRow;
use std::path::Path;

// Governed input filenames are case-sensitive corpus contracts.
#[expect(clippy::case_sensitive_file_extension_comparisons)]
pub(crate) fn resolve_governed_vcf_call_bam_smoke_contract(
    inputs: &GenotypeBenchmarkInputs,
    repo_root: &Path,
    requested_stage_id: &str,
    requested_tool_id: &str,
    expected_output_id: &str,
) -> Result<GovernedVcfCallBamSmokeContract> {
    let matrix_row = resolve_vcf_call_matrix_row(
        inputs,
        requested_stage_id,
        requested_tool_id,
        expected_output_id,
    )?;
    let fixture_report = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH),
    )?;
    let sample = fixture_report
        .samples
        .iter()
        .find(|sample| sample.sample_id == GOVERNED_BAM_SAMPLE_ID)
        .ok_or_else(|| {
            anyhow!(
                "governed BAM fixture is missing required VCF call smoke sample `{GOVERNED_BAM_SAMPLE_ID}`"
            )
        })?;
    let sample_name = sample.observed_header_sample_ids.first().cloned().ok_or_else(|| {
        anyhow!("governed BAM fixture sample `{}` has no observed sample name", sample.sample_id)
    })?;
    if !sample.alignment_path.ends_with(".bam") {
        bail!(
            "governed VCF BAM smoke requires a real BAM input, found `{}`",
            sample.alignment_path
        );
    }
    if !sample.index_path.ends_with(".bam.bai") {
        bail!("governed VCF BAM smoke requires a BAM index sidecar, found `{}`", sample.index_path);
    }

    Ok(GovernedVcfCallBamSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_bam: sample.alignment_path.clone(),
        input_bam_index: sample.index_path.clone(),
        reference: fixture_report.reference_fasta,
        sample_id: sample.sample_id.clone(),
        sample_name,
    })
}

pub(crate) fn resolve_vcf_call_matrix_row(
    inputs: &GenotypeBenchmarkInputs,
    requested_stage_id: &str,
    requested_tool_id: &str,
    expected_output_id: &str,
) -> Result<VcfStageMatrixRow> {
    let matrix_row = inputs
        .rows()
        .find(|&row| row.stage_id == requested_stage_id)
        .cloned()
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{requested_stage_id}`"))?;
    if matrix_row.tool_id != GOVERNED_VCF_CALL_TOOL_ID {
        bail!(
            "VCF BAM smoke requires retained tool `{GOVERNED_VCF_CALL_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF BAM smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != "vcf_production_regression" {
        bail!(
            "VCF BAM smoke requires corpus `vcf_production_regression`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != "bam_bundle" {
        bail!(
            "VCF BAM smoke requires asset profile `bam_bundle`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec![expected_output_id.to_string()] {
        bail!(
            "VCF BAM smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(matrix_row)
}
