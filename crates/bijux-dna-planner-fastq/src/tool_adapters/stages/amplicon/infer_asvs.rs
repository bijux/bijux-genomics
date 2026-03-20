use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::ExecutionStatus;
use bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_INFER_ASVS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    match bijux_dna_domain_fastq::execution_support_for_stage(&STAGE_ID) {
        Some(support) if support.execution_status == ExecutionStatus::Closed => {}
        _ => {
            return Err(anyhow!(
                "{} is declared-only and cannot be planned until its runtime contract closes",
                STAGE_ID.as_str()
            ));
        }
    }
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    let asv_table = out_dir.join("asv_abundance.tsv");
    let asv_sequences = out_dir.join("asv_sequences.fasta");
    let taxonomy_ready_fasta = out_dir.join("taxonomy_ready.fasta");
    let taxonomy_ready_fastq = out_dir.join("taxonomy_ready.fastq");
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: infer_asvs_command(
                &tool.tool_id.0,
                r1,
                r2,
                &asv_table,
                &asv_sequences,
                &taxonomy_ready_fasta,
                &taxonomy_ready_fastq,
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("asv_table_tsv"),
                    asv_table.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("asv_sequences_fasta"),
                    asv_sequences.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fasta"),
                    taxonomy_ready_fasta.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fastq"),
                    taxonomy_ready_fastq.clone(),
                    ArtifactRole::Reads,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({
            "requires_r_runtime": true,
            "output_table_kind": "asv_abundance_table",
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
            "runtime_constraints": {
                "requires_r": true,
                "min_r_major": 4
            }
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon ASV inference",
        ),
    })
}

fn infer_asvs_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    asv_table: &Path,
    asv_sequences: &Path,
    taxonomy_ready_fasta: &Path,
    taxonomy_ready_fastq: &Path,
) -> Result<Vec<String>> {
    match tool_id {
        "dada2" => {
            let mut command = vec![
                "Rscript".to_string(),
                "run_dada2.R".to_string(),
                "--input-r1".to_string(),
                r1.display().to_string(),
            ];
            if let Some(r2) = r2 {
                command.push("--input-r2".to_string());
                command.push(r2.display().to_string());
            }
            command.extend([
                "--asv-table".to_string(),
                asv_table.display().to_string(),
                "--asv-fasta".to_string(),
                asv_sequences.display().to_string(),
                "--taxonomy-ready-fasta".to_string(),
                taxonomy_ready_fasta.display().to_string(),
                "--taxonomy-ready-fastq".to_string(),
                taxonomy_ready_fastq.display().to_string(),
            ]);
            Ok(command)
        }
        _ => Err(anyhow!(
            "unsupported ASV inference tool for stage planning: {tool_id}"
        )),
    }
}
