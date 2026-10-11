//! call admission bindings.
use crate::bam::admission::bindings::resolve_governed_vcf_call_bam_smoke_contract;
use crate::call::model::GovernedVcfCallSmokeContract;
use crate::call::model::GOVERNED_VCF_CALL_RESOLVED_STAGE_ID;
use crate::call::model::GOVERNED_VCF_CALL_STAGE_ID;
use crate::inputs::GenotypeBenchmarkInputs;
use anyhow::Result;
use std::path::Path;

pub(crate) fn resolve_governed_vcf_call_smoke_contract(
    inputs: &GenotypeBenchmarkInputs,
    repo_root: &Path,
    requested_tool_id: &str,
) -> Result<GovernedVcfCallSmokeContract> {
    let base_contract = resolve_governed_vcf_call_bam_smoke_contract(
        inputs,
        repo_root,
        GOVERNED_VCF_CALL_STAGE_ID,
        requested_tool_id,
        "called_vcf",
    )?;
    Ok(GovernedVcfCallSmokeContract {
        stage_id: base_contract.stage_id,
        resolved_stage_id: GOVERNED_VCF_CALL_RESOLVED_STAGE_ID.to_string(),
        tool_id: base_contract.tool_id,
        corpus_id: base_contract.corpus_id,
        input_bam: base_contract.input_bam,
        input_bam_index: base_contract.input_bam_index,
        reference: base_contract.reference,
        sample_id: base_contract.sample_id,
        sample_name: base_contract.sample_name,
    })
}
