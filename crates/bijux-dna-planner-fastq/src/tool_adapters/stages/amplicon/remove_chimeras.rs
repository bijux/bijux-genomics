use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_CHIMERAS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_REMOVE_CHIMERAS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(tool: &ToolExecutionSpecV1, reads: &Path, out_dir: &Path) -> Result<StagePlanV1> {
    let filtered = out_dir.join("nonchimeras.fastq.gz");
    let metrics = out_dir.join("chimera_metrics.json");
    let chimeras = out_dir.join("chimeras.fasta");
    let uchime = out_dir.join("uchime.tsv");
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: vec![
                "vsearch".to_string(),
                "--uchime_denovo".to_string(),
                "{{reads}}".to_string(),
                "--nonchimeras".to_string(),
                filtered.to_string_lossy().to_string(),
                "--chimeras".to_string(),
                chimeras.to_string_lossy().to_string(),
                "--uchimeout".to_string(),
                uchime.to_string_lossy().to_string(),
            ],
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads"),
                reads.to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("chimera_filtered_reads"),
                    filtered,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("chimera_metrics_json"),
                    metrics,
                    ArtifactRole::MetricsJson,
                ),
                ArtifactRef::optional(
                    ArtifactId::from_static("chimeras_fasta"),
                    chimeras,
                    ArtifactRole::Index,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({
            "chimera_mode": "denovo",
            "report_auxiliary_uchime_table": uchime,
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon chimera removal",
        ),
    })
}
