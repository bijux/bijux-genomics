use anyhow::Result;
use bijux_core::contract::canonical::parameters_json_canonicalization;
use bijux_core::contract::ContractVersion;
use bijux_core::foundation::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_core::metrics::MetricsEnvelope;
use bijux_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_stage_contract::{StageInvocationV1, StagePlugin, StagePluginOutputV1};

use crate::metrics;

#[allow(dead_code)]
pub struct FastqStagePlugin;

impl StagePlugin for FastqStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with("fastq.")
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
        _outputs: &[ArtifactRef],
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
        let metrics = metrics::stage_metrics_for_plan(plan, &input_paths, &output_paths)?;
        let mut input_hashes = Vec::new();
        for path in &input_paths {
            if path.exists() {
                if let Ok(hash) = bijux_infra::hash_file_sha256(path) {
                    input_hashes.push(hash);
                }
            }
        }
        let input_fingerprint = input_fingerprint(&input_hashes);
        let parameters_fingerprint = parameters_fingerprint(&plan.params)?;
        let parameters_json_normalized = parameters_json_canonicalization(&plan.params);
        let image_digest = plan
            .image
            .digest
            .clone()
            .unwrap_or_else(|| plan.image.image.clone());
        let envelope = MetricsEnvelope {
            schema_version: "bijux.metrics_envelope.v2".to_string(),
            contract_version: ContractVersion::v1(),
            stage_id: plan.stage_id.0.to_string(),
            stage_version: plan.stage_version.0,
            tool_id: plan.tool_id.0.to_string(),
            tool_version: plan.tool_version.clone(),
            image_digest,
            parameters_fingerprint,
            input_fingerprint,
            parameters_json_normalized,
            input_hashes,
            metrics,
        };
        Ok(StagePluginOutputV1 {
            metrics: envelope,
            artifacts: Vec::new(),
            report_parts: Vec::new(),
            warnings: Vec::new(),
            invariants: Vec::new(),
            verdict: None,
            event_hints: Vec::new(),
        })
    }
}
