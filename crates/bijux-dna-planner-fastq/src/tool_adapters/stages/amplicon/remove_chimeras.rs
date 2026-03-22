use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::edna::ChimeraDetectionEffectiveParams;
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
    if r2.is_some() {
        return Err(anyhow!(
            "vsearch chimera removal requires a single merged or single-end input stream"
        ));
    }
    let filtered_reads = out_dir.join("nonchimeras.fastq.gz");
    let report = out_dir.join("remove_chimeras_report.json");
    let metrics = out_dir.join("chimera_metrics.json");
    let chimeras = out_dir.join("chimeras.fasta");
    let uchime = out_dir.join("uchime.tsv");
    let inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("chimera_filtered_reads"),
        filtered_reads.clone(),
        ArtifactRole::Reads,
    )];
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("chimera_metrics_json"),
        metrics.clone(),
        ArtifactRole::MetricsJson,
    ));
    outputs.push(ArtifactRef::optional(
        ArtifactId::from_static("chimeras_fasta"),
        chimeras.clone(),
        ArtifactRole::Reads,
    ));
    outputs.push(ArtifactRef::optional(
        ArtifactId::from_static("uchime_report_tsv"),
        uchime.clone(),
        ArtifactRole::SummaryTsv,
    ));
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: vec![
                "vsearch".to_string(),
                "--uchime_denovo".to_string(),
                r1.to_string_lossy().to_string(),
                "--nonchimeras".to_string(),
                filtered_reads.to_string_lossy().to_string(),
                "--chimeras".to_string(),
                chimeras.to_string_lossy().to_string(),
                "--uchimeout".to_string(),
                uchime.to_string_lossy().to_string(),
            ],
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "input_reads": r1,
            "chimera_filtered_reads": filtered_reads,
            "report_json": report,
            "chimera_metrics_json": metrics,
            "chimeras_fasta": chimeras,
            "uchime_report_tsv": uchime,
        }),
        effective_params: serde_json::to_value(ChimeraDetectionEffectiveParams {
            method: "vsearch_uchime_denovo".to_string(),
            detection_scope: "denovo".to_string(),
            input_layout: "single_stream".to_string(),
            report_artifact: "report_json".to_string(),
            metrics_artifact: "chimera_metrics_json".to_string(),
            chimera_sequence_artifact: "chimeras_fasta".to_string(),
            raw_backend_report_artifact: "uchime_report_tsv".to_string(),
            raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
            chimera_removed_definition:
                "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                    .to_string(),
            fallback_behavior: "copy_input_reads_and_mark_report".to_string(),
        })
        .map_err(|error| anyhow!("serialize chimera effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "amplicon chimera removal"),
    })
}
