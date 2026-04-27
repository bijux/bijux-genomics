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

/// # Errors
/// Returns an error if chimera removal cannot be planned for the requested tool or input layout.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let effective_params = default_effective_params(tool.resources.threads.max(1));
    plan_with_effective_params(tool, r1, r2, out_dir, &effective_params)
}

/// # Errors
/// Returns an error if the requested chimera-detection parameters are unsupported or the stage
/// plan cannot be built.
pub fn plan_with_effective_params(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    effective_params: &ChimeraDetectionEffectiveParams,
) -> Result<StagePlanV1> {
    if r2.is_some() {
        return Err(anyhow!(
            "vsearch chimera removal requires a single merged or single-end input stream"
        ));
    }
    let effective_params = normalize_effective_params(effective_params)?;
    let threads = effective_params.threads;
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
                "--threads".to_string(),
                threads.to_string(),
            ],
        },
        resources: {
            let mut resources = tool.resources.clone();
            resources.threads = threads;
            resources
        },
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "input_reads": r1,
            "threads": threads,
            "method": effective_params.method.clone(),
            "detection_scope": effective_params.detection_scope.clone(),
            "input_layout": effective_params.input_layout.clone(),
            "chimera_filtered_reads": filtered_reads,
            "report_json": report,
            "chimera_metrics_json": metrics,
            "chimeras_fasta": chimeras,
            "uchime_report_tsv": uchime,
            "raw_backend_report_artifact": effective_params.raw_backend_report_artifact.clone(),
            "raw_backend_report_format": effective_params.raw_backend_report_format.clone(),
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize chimera effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "amplicon chimera removal"),
    })
}

fn default_effective_params(threads: u32) -> ChimeraDetectionEffectiveParams {
    ChimeraDetectionEffectiveParams {
        method: "vsearch_uchime_denovo".to_string(),
        detection_scope: "denovo".to_string(),
        input_layout: "single_stream".to_string(),
        threads,
        report_artifact: "report_json".to_string(),
        metrics_artifact: "chimera_metrics_json".to_string(),
        chimera_sequence_artifact: "chimeras_fasta".to_string(),
        raw_backend_report_artifact: "uchime_report_tsv".to_string(),
        raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
        chimera_removed_definition:
            "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                .to_string(),
        fallback_behavior: "copy_input_reads_and_mark_report".to_string(),
    }
}

fn normalize_effective_params(
    effective_params: &ChimeraDetectionEffectiveParams,
) -> Result<ChimeraDetectionEffectiveParams> {
    let normalized = ChimeraDetectionEffectiveParams {
        threads: effective_params.threads.max(1),
        ..effective_params.clone()
    };
    let expected = default_effective_params(normalized.threads);
    for (field, actual, supported) in [
        ("method", normalized.method.as_str(), expected.method.as_str()),
        ("detection_scope", normalized.detection_scope.as_str(), expected.detection_scope.as_str()),
        ("input_layout", normalized.input_layout.as_str(), expected.input_layout.as_str()),
        ("report_artifact", normalized.report_artifact.as_str(), expected.report_artifact.as_str()),
        (
            "metrics_artifact",
            normalized.metrics_artifact.as_str(),
            expected.metrics_artifact.as_str(),
        ),
        (
            "chimera_sequence_artifact",
            normalized.chimera_sequence_artifact.as_str(),
            expected.chimera_sequence_artifact.as_str(),
        ),
        (
            "raw_backend_report_artifact",
            normalized.raw_backend_report_artifact.as_str(),
            expected.raw_backend_report_artifact.as_str(),
        ),
        (
            "raw_backend_report_format",
            normalized.raw_backend_report_format.as_str(),
            expected.raw_backend_report_format.as_str(),
        ),
        (
            "chimera_removed_definition",
            normalized.chimera_removed_definition.as_str(),
            expected.chimera_removed_definition.as_str(),
        ),
        (
            "fallback_behavior",
            normalized.fallback_behavior.as_str(),
            expected.fallback_behavior.as_str(),
        ),
    ] {
        if actual != supported {
            return Err(anyhow!(
                "fastq.remove_chimeras only supports {field}={supported}, got {actual}"
            ));
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::plan_with_effective_params;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
        ToolVersion,
    };
    use bijux_dna_domain_fastq::params::edna::ChimeraDetectionEffectiveParams;
    use std::path::Path;

    fn vsearch_tool() -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("vsearch"),
            tool_version: ToolVersion::from("2.29.0"),
            image: ContainerImageRefV1 { image: "bijuxdna/vsearch".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["vsearch".to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 2,
            },
        }
    }

    #[test]
    fn remove_chimeras_plan_honors_effective_thread_override() {
        let plan = plan_with_effective_params(
            &vsearch_tool(),
            Path::new("merged.fastq.gz"),
            None,
            Path::new("out"),
            &ChimeraDetectionEffectiveParams {
                method: "vsearch_uchime_denovo".to_string(),
                detection_scope: "denovo".to_string(),
                input_layout: "single_stream".to_string(),
                threads: 7,
                report_artifact: "report_json".to_string(),
                metrics_artifact: "chimera_metrics_json".to_string(),
                chimera_sequence_artifact: "chimeras_fasta".to_string(),
                raw_backend_report_artifact: "uchime_report_tsv".to_string(),
                raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
                chimera_removed_definition:
                    "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                        .to_string(),
                fallback_behavior: "copy_input_reads_and_mark_report".to_string(),
            },
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 7);
        assert!(
            plan.command.template.windows(2).any(|pair| pair == ["--threads", "7"]),
            "vsearch command must render the governed thread override",
        );
    }
}
