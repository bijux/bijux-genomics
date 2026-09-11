use anyhow::{ensure, Result};
use bijux_dna_bench_vcf_evidence::{
    model::pca::PcaAdapterEvidence,
    snapshot::{VcfStageReadinessBinding, VcfStageReadinessBindingKey},
};
use std::collections::BTreeMap;
pub(super) fn admit_adapter_evidence<'a>(
    bindings: &[VcfStageReadinessBinding],
    evidence: &'a [PcaAdapterEvidence],
) -> Result<BTreeMap<VcfStageReadinessBindingKey, &'a PcaAdapterEvidence>> {
    let mut admitted = BTreeMap::new();
    for observation in evidence {
        ensure!(
            observation.binding.stage_id == "vcf.pca"
                && ["plink2", "eigensoft"].contains(&observation.binding.tool_id.as_str()),
            "PCA readiness received foreign adapter evidence"
        );
        let binding = bindings
            .iter()
            .find(|binding| key(binding) == observation.binding)
            .ok_or_else(|| anyhow::anyhow!("PCA adapter evidence has no retained full binding"))?;
        let row = &binding.retained_row;
        ensure!(
            observation.adapter_id == row.adapter_id
                && observation.parser_id == row.parser_id
                && observation.schema_id == row.schema_id,
            "PCA adapter schema or parser disagrees with retained evidence"
        );
        let output = &observation.output;
        ensure!(
            !output.output_proof_path.is_empty()
                && output.output_proof_path.trim() == output.output_proof_path,
            "PCA output publication address is empty or padded"
        );
        ensure!(
            observation.command.stage_id == row.stage_id
                && observation.command.tool_id == row.tool_id,
            "PCA command identity disagrees with retained binding"
        );
        if let Some(command) = &binding.command_row {
            ensure!(
                command.command_steps == observation.command.command_steps
                    && command.script_commands == observation.command.script_commands
                    && command.command_source == observation.command.command_source
                    && command.benchmark_status == observation.command.benchmark_status,
                "PCA command observations disagree with readiness snapshot"
            );
        }
        ensure!(
            admitted.insert(observation.binding.clone(), observation).is_none(),
            "PCA readiness received repeated adapter evidence"
        );
    }
    ensure!(
        bindings.iter().all(|binding| admitted.contains_key(&key(binding))),
        "PCA readiness is missing retained adapter output evidence"
    );
    Ok(admitted)
}
pub(super) fn key(binding: &VcfStageReadinessBinding) -> VcfStageReadinessBindingKey {
    let row = &binding.retained_row;
    VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}
