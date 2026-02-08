use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_core::contract::canonical::parameters_json_canonicalization;
use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{StageInvocationV1, StagePlugin, StagePluginOutputV1};

use crate::metrics::bam_metrics_from_dir;

#[allow(dead_code)]
pub struct BamStagePlugin;

impl StagePlugin for BamStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with("bam.")
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        Ok(StageInvocationV1 {
            command: plan.command.template.clone(),
            env: BTreeMap::new(),
            expected_outputs: plan.io.outputs.clone(),
        })
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        let out_dir = outputs
            .first()
            .and_then(|output| output.path.parent())
            .map_or_else(|| std::path::PathBuf::from("."), std::path::PathBuf::from);
        let mut metrics = bam_metrics_from_dir(&out_dir);
        let thresholds = bijux_dna_domain_bam::metrics::BamInvariantThresholds::default();
        let evaluation = bijux_dna_domain_bam::metrics::evaluate_bam_invariants(
            &plan.stage_id.0,
            &metrics,
            &thresholds,
        );
        metrics.stage_verdict = Some(evaluation.verdict.into());
        let metrics_json = serde_json::to_value(metrics)?;
        let mut input_hashes = Vec::new();
        for input in &plan.io.inputs {
            if input.path.exists() {
                if let Ok(hash) = bijux_dna_infra::hash_file_sha256(&input.path) {
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
            metrics: metrics_json,
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
