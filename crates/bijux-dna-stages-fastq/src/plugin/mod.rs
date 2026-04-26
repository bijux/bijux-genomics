use anyhow::{anyhow, Result};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{StageInvocationV1, StagePlugin, StagePluginOutputV1};
use std::fs;

use crate::metrics;
use crate::observer::{
    parse_bbduk_reads_removed, parse_cluster_otus_report, parse_correct_errors_report,
    parse_deduplicate_report, parse_deplete_host_report,
    parse_deplete_reference_contaminants_report, parse_deplete_rrna_report,
    parse_detect_adapters_report, parse_extract_umis_report, parse_fastp_metrics,
    parse_filter_low_complexity_report, parse_filter_reads_report, parse_index_reference_report,
    parse_infer_asvs_report, parse_merge_pairs_report, parse_multiqc_general_stats_metrics,
    parse_normalize_abundance_report, parse_normalize_primers_report,
    parse_profile_overrepresented_report, parse_profile_read_lengths_report,
    parse_profile_reads_report, parse_remove_chimeras_report, parse_remove_duplicates_provenance,
    parse_remove_duplicates_report, parse_report_qc_report, parse_screen_taxonomy_report,
    parse_terminal_damage_report, parse_trim_polyg_report, parse_trim_reads_report,
    parse_validated_reads_manifest, parse_validation_report,
};

mod observation_context;
mod output_contract;
mod semantic;

#[allow(dead_code)]
pub struct FastqStagePlugin;

impl StagePlugin for FastqStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        bijux_dna_domain_fastq::STAGES.iter().any(|stage| stage.as_str() == stage_id)
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        if !self.handles_stage(plan.stage_id.as_str()) {
            return Err(anyhow!("unsupported FASTQ stage {}", plan.stage_id.as_str()));
        }
        if plan.command.template.is_empty() {
            return Err(anyhow!(
                "FASTQ stage {} has empty command template",
                plan.stage_id.as_str()
            ));
        }
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
        let input_paths: Vec<std::path::PathBuf> =
            plan.io.inputs.iter().map(|input| input.path.clone()).collect();
        let output_refs = if outputs.is_empty() { &plan.io.outputs } else { outputs };
        let output_paths: Vec<std::path::PathBuf> =
            output_refs.iter().map(|output| output.path.clone()).collect();
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
            report_parts,
            warnings,
            invariants,
            verdict: Some(verdict),
            event_hints,
        })
    }
}

#[cfg(test)]
mod plugin_contracts;
