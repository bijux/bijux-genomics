use super::{
    commands::{vcf_pca_eigensoft_command_row, vcf_pca_plink_command_row},
    outputs::{eigensoft_output_proof, plink_output_proof},
    PreparedPcaAdapterEvidence,
};
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{ensure, Result};
use bijux_dna_bench_vcf_evidence::{
    model::pca::PcaAdapterEvidence, snapshot::VcfStageReadinessBindingKey,
};
/// Acquire PCA command/output observations under unambiguous admitted binding declarations.
///
/// # Errors
/// Refuses missing or ambiguous PCA declarations before acquisition, or invalid family reports.
pub fn prepare_pca_adapter_evidence(
    observations: &VcfAdapterObservations<'_>,
) -> Result<PreparedPcaAdapterEvidence> {
    let mut declarations = Vec::new();
    for tool in ["plink2", "eigensoft"] {
        let rows = observations
            .inputs()
            .bindings()
            .iter()
            .filter(|row| row.stage_id == "vcf.pca" && row.tool_id == tool)
            .collect::<Vec<_>>();
        ensure!(
            rows.len() == 1,
            "PCA evidence requires exactly one full binding for each retained adapter"
        );
        declarations.push(rows[0]);
    }
    let schemas=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors()?;
    let schema = schemas
        .iter()
        .find(|row| row.stage_id == "vcf.pca")
        .ok_or_else(|| anyhow::anyhow!("PCA evidence has no normalized schema"))?;
    let mut evidence = Vec::new();
    for declaration in declarations {
        let (output, command) = match declaration.tool_id.as_str() {
            "plink2" => {
                let report = observations.plink("plink2")?;
                let rows =
                    report.rows.iter().filter(|row| row.stage_id == "vcf.pca").collect::<Vec<_>>();
                ensure!(rows.len() == 1, "PCA evidence requires one PLINK2 adapter observation");
                (plink_output_proof(rows[0]), vcf_pca_plink_command_row(rows[0].clone()))
            }
            "eigensoft" => {
                let report = observations.eigensoft()?;
                let rows =
                    report.rows.iter().filter(|row| row.stage_id == "vcf.pca").collect::<Vec<_>>();
                ensure!(rows.len() == 1, "PCA evidence requires one EIGENSOFT adapter observation");
                (eigensoft_output_proof(rows[0]), vcf_pca_eigensoft_command_row(rows[0].clone()))
            }
            _ => anyhow::bail!("PCA evidence received an unowned adapter"),
        };
        ensure!(
            command.stage_id == declaration.stage_id && command.tool_id == declaration.tool_id,
            "PCA adapter identity disagrees with its admitted binding"
        );
        evidence.push(PcaAdapterEvidence {
            binding: VcfStageReadinessBindingKey {
                stage_id: declaration.stage_id.clone(),
                tool_id: declaration.tool_id.clone(),
                corpus_id: declaration.corpus_id.clone(),
                asset_profile_id: declaration.asset_profile_id.clone(),
            },
            adapter_id: declaration.adapter_id.clone(),
            parser_id: declaration.parser_id.clone(),
            schema_id: schema.schema_id.clone(),
            output,
            command,
        });
    }
    evidence.sort_by(|left, right| left.binding.tool_id.cmp(&right.binding.tool_id));
    Ok(PreparedPcaAdapterEvidence { rows: evidence })
}
