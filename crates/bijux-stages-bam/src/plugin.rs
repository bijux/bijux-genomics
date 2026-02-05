use std::collections::BTreeMap;

use anyhow::Result;
use bijux_core::plan::stage_plugin::{StageInvocationV1, StagePlugin, StagePluginOutputV1};
use bijux_core::{ArtifactRef, StagePlanV1};

use crate::metrics::bam_metrics_from_dir;

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
        let thresholds = bijux_domain_bam::metrics::BamInvariantThresholds::default();
        let evaluation = bijux_domain_bam::metrics::evaluate_bam_invariants(
            &plan.stage_id.0,
            &metrics,
            &thresholds,
        );
        metrics.stage_verdict = Some(evaluation.verdict.into());
        Ok(StagePluginOutputV1 {
            metrics: serde_json::to_value(metrics)?,
            artifacts: Vec::new(),
            report_parts: Vec::new(),
            warnings: Vec::new(),
            invariants: Vec::new(),
            verdict: None,
            event_hints: Vec::new(),
        })
    }
}
