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

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let filtered_r1 = if r2.is_some() {
        out_dir.join("nonchimeras_R1.fastq.gz")
    } else {
        out_dir.join("nonchimeras.fastq.gz")
    };
    let filtered_r2 = r2.map(|_| out_dir.join("nonchimeras_R2.fastq.gz"));
    let metrics = out_dir.join("chimera_metrics.json");
    let chimeras = out_dir.join("chimeras.fasta");
    let uchime = out_dir.join("uchime.tsv");
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
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("chimera_filtered_reads_r1"),
        filtered_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(filtered_r2) = &filtered_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("chimera_filtered_reads_r2"),
            filtered_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("chimera_metrics_json"),
        metrics.clone(),
        ArtifactRole::MetricsJson,
    ));
    outputs.push(ArtifactRef::optional(
        ArtifactId::from_static("chimeras_fasta"),
        chimeras.clone(),
        ArtifactRole::Index,
    ));
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
                "{{reads_r1}}".to_string(),
                "--nonchimeras".to_string(),
                filtered_r1.to_string_lossy().to_string(),
                "--chimeras".to_string(),
                chimeras.to_string_lossy().to_string(),
                "--uchimeout".to_string(),
                uchime.to_string_lossy().to_string(),
            ],
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({
            "chimera_mode": "denovo",
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
            "report_auxiliary_uchime_table": uchime,
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon chimera removal",
        ),
    })
}
