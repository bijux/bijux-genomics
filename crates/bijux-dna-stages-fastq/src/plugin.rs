use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{
    StageEventHintV1, StageInvocationV1, StagePlugin, StagePluginOutputV1, StageReportPartV1,
};

use crate::metrics;

#[allow(dead_code)]
pub struct FastqStagePlugin;

impl StagePlugin for FastqStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with(id_catalog::FASTQ_PREFIX)
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        Ok(StageInvocationV1 {
            command: plan.command.template.clone(),
            env: std::collections::BTreeMap::new(),
            expected_outputs: plan.io.outputs.clone(),
        })
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        let input_paths: Vec<std::path::PathBuf> = plan
            .io
            .inputs
            .iter()
            .map(|input| input.path.clone())
            .collect();
        let output_paths: Vec<std::path::PathBuf> = plan
            .io
            .outputs
            .iter()
            .map(|output| output.path.clone())
            .collect();
        let envelope = metrics::build_metrics_envelope(plan, &input_paths, &output_paths)?;
        let observer_covered = crate::observer_stage_ids()
            .into_iter()
            .any(|stage_id| stage_id == plan.stage_id);
        let artifacts = if outputs.is_empty() {
            plan.io.outputs.clone()
        } else {
            outputs.to_vec()
        };
        let report_parts = vec![StageReportPartV1 {
            name: "stage_outputs".to_string(),
            file_name: "stage_outputs.json".to_string(),
            payload: serde_json::json!({
                "stage_id": plan.stage_id,
                "observer_coverage": observer_covered,
                "artifact_count": artifacts.len(),
                "artifact_ids": artifacts
                    .iter()
                    .map(|artifact| artifact.name.as_str().to_string())
                    .collect::<Vec<_>>(),
                "used_observed_outputs": !outputs.is_empty(),
            }),
        }];
        let warnings = if observer_covered {
            Vec::new()
        } else {
            vec![format!(
                "{} has no observer-specialized parser; emitting metrics envelope from the stage plan only",
                plan.stage_id.as_str()
            )]
        };
        let event_hints = vec![StageEventHintV1 {
            event_name: "stage_outputs_parsed".to_string(),
            status: if outputs.is_empty() {
                "expected_only".to_string()
            } else {
                "observed".to_string()
            },
            attrs: serde_json::json!({
                "stage_id": plan.stage_id,
                "observer_coverage": observer_covered,
                "artifact_count": artifacts.len(),
            }),
        }];
        Ok(StagePluginOutputV1 {
            metrics: envelope,
            artifacts,
            report_parts,
            warnings,
            invariants: Vec::new(),
            verdict: None,
            event_hints,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bijux_dna_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
    use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StagePlugin};

    use super::FastqStagePlugin;

    fn plan(stage_id: &str) -> bijux_dna_stage_contract::StagePlanV1 {
        bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::new(stage_id),
            stage_version: StageVersion(1),
            tool_id: ToolId::new("fastqc"),
            tool_version: "test".to_string(),
            image: ContainerImageRefV1 {
                image: "bijuxdna/test".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["echo".to_string(), "ok".to_string()],
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    PathBuf::from("reads.fastq.gz"),
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    PathBuf::from("report.json"),
                    ArtifactRole::ReportJson,
                )],
            },
            out_dir: PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        }
    }

    #[test]
    fn parse_outputs_emits_artifacts_report_parts_and_event_hints() {
        let plugin = FastqStagePlugin;
        let plan = plan("fastq.detect_adapters");
        let output = plugin
            .parse_outputs(&plan, &plan.io.outputs)
            .expect("parse outputs");
        assert_eq!(output.artifacts.len(), 1);
        assert_eq!(output.report_parts.len(), 1);
        assert_eq!(output.event_hints.len(), 1);
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn parse_outputs_warns_when_no_observer_parser_exists() {
        let plugin = FastqStagePlugin;
        let plan = plan("fastq.trim_reads");
        let output = plugin
            .parse_outputs(&plan, &plan.io.outputs)
            .expect("parse outputs");
        assert_eq!(output.artifacts.len(), 1);
        assert_eq!(output.warnings.len(), 1);
        assert!(output.warnings[0].contains("fastq.trim_reads"));
    }
}
