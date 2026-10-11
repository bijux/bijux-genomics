//! Owner: bijux-dna-bench-population
//! Ibd admission binding.
use crate::ibd::model::{
    GovernedVcfIbdSmokeContract, GOVERNED_VCF_IBD_ASSET_PROFILE_ID, GOVERNED_VCF_IBD_CORPUS_ID,
    GOVERNED_VCF_IBD_INPUT_FIXTURE_ID, GOVERNED_VCF_IBD_STAGE_ID,
};
use crate::inputs::PopulationBenchmarkInputs;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::path::Path;

pub(crate) fn resolve_governed_vcf_ibd_smoke_contract(
    inputs: &PopulationBenchmarkInputs,
    _repo_root: &Path,
    tool_id: &str,
) -> Result<GovernedVcfIbdSmokeContract> {
    let matrix_row = inputs
        .governed_rows()
        .iter()
        .find(|row| row.stage_id == GOVERNED_VCF_IBD_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_IBD_STAGE_ID}`"))?;
    let mut active_rows = inputs.active_ibd_rows().iter().filter(|row| {
        row.domain == "vcf" && row.stage_id == GOVERNED_VCF_IBD_STAGE_ID && row.tool_id == tool_id
    });
    let active_row = active_rows.next().ok_or_else(|| {
        anyhow!(
            "VCF IBD smoke requires an active `{GOVERNED_VCF_IBD_STAGE_ID}` binding; requested `{tool_id}`"
        )
    })?;
    if active_rows.next().is_some() {
        bail!("VCF IBD smoke active binding is not unique for `{tool_id}`");
    }
    if active_row.status != "benchmark_ready" {
        bail!(
            "VCF IBD smoke requires benchmark-ready status for `{tool_id}`, found `{}`",
            active_row.status
        );
    }
    if active_row.corpus_id != GOVERNED_VCF_IBD_CORPUS_ID {
        bail!(
            "VCF IBD smoke requires corpus `{GOVERNED_VCF_IBD_CORPUS_ID}`, found `{}`",
            active_row.corpus_id
        );
    }
    if active_row.asset_profile_id != GOVERNED_VCF_IBD_ASSET_PROFILE_ID {
        bail!(
            "VCF IBD smoke requires asset profile `{GOVERNED_VCF_IBD_ASSET_PROFILE_ID}`, found `{}`",
            active_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["ibd_segments".to_string()] {
        bail!(
            "VCF IBD smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfIbdSmokeContract {
        stage: matrix_row.stage_id.clone(),
        tool: active_row.tool_id.clone(),
        corpus: active_row.corpus_id.clone(),
        input_fixture: GOVERNED_VCF_IBD_INPUT_FIXTURE_ID.to_string(),
    })
}
