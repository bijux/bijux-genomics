use anyhow::{anyhow, ensure, Result};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{StageInvocationV1, StagePlugin, StagePluginOutputV1};

use crate::metrics;

mod observation_context;
mod output_contract;
mod semantic;

pub struct FastqStagePlugin;

impl StagePlugin for FastqStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        bijux_dna_domain_fastq::STAGES.iter().any(|stage| stage.as_str() == stage_id)
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        if !self.handles_stage(plan.stage_id.as_str()) {
            return Err(anyhow!("unsupported FASTQ stage {}", plan.stage_id.as_str()));
        }
        validate_command_template(plan)?;
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
        if !self.handles_stage(plan.stage_id.as_str()) {
            return Err(anyhow!("unsupported FASTQ stage {}", plan.stage_id.as_str()));
        }
        let output_refs = if outputs.is_empty() { &plan.io.outputs } else { outputs };
        let input_paths: Vec<std::path::PathBuf> = plan
            .io
            .inputs
            .iter()
            .filter(|input| input.role == bijux_dna_core::contract::ArtifactRole::Reads)
            .map(|input| input.path.clone())
            .collect();
        let output_paths: Vec<std::path::PathBuf> = output_refs
            .iter()
            .filter(|output| output.role == bijux_dna_core::contract::ArtifactRole::Reads)
            .map(|output| output.path.clone())
            .collect();
        let envelope = metrics::build_metrics_envelope(plan, &input_paths, &output_paths)?;
        let context = observation_context::observation_context(plan, outputs);
        let expected_artifact_names = plan
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.name.as_str().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        let actual_artifact_names = context
            .artifacts
            .iter()
            .map(|artifact| artifact.name.as_str().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        let missing_expected =
            expected_artifact_names.difference(&actual_artifact_names).cloned().collect::<Vec<_>>();
        let outputs_used = !outputs.is_empty();
        let invariants = output_contract::output_invariants(&missing_expected, &context);
        let verdict = output_contract::output_verdict(plan, outputs_used, &invariants, &context);
        let report_parts = output_contract::output_report_parts(plan, outputs_used, &context);
        let warnings = output_contract::output_warnings(plan, &context);
        let event_hints = output_contract::output_event_hints(plan, outputs_used, &context);
        Ok(StagePluginOutputV1 {
            metrics: envelope,
            artifacts: context.artifacts,
            operating_mode: plan.operating_mode,
            report_parts,
            warnings,
            invariants,
            verdict: Some(verdict),
            event_hints,
        })
    }
}

fn validate_command_template(plan: &StagePlanV1) -> Result<()> {
    ensure!(
        !plan.command.template.is_empty(),
        "FASTQ stage {} has empty command template",
        plan.stage_id.as_str()
    );
    ensure!(
        plan.command.template.iter().all(|arg| !arg.trim().is_empty()),
        "FASTQ stage {} has blank command template argument",
        plan.stage_id.as_str()
    );
    Ok(())
}

#[cfg(test)]
mod plugin_contracts;
