pub mod markdup {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::MarkDupEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Markdup.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &MarkDupEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Markdup, out_dir);
        let out_bam = out_dir.join("markdup.bam");
        let flagstat_before = out_dir.join("flagstat.before.txt");
        let flagstat_after = out_dir.join("flagstat.after.txt");
        let idxstats_before = out_dir.join("idxstats.before.txt");
        let idxstats_after = out_dir.join("idxstats.after.txt");
        let summary = out_dir.join("markdup.summary.json");
        let command = match tool.tool_id.0.as_str() {
            "samtools" => crate::tools::samtools::markdup_args_with_audit(
                bam,
                &out_bam,
                &flagstat_before,
                &flagstat_after,
                &idxstats_before,
                &idxstats_after,
                &summary,
                params,
            ),
            _ => crate::tools::gatk::markdup_args_with_audit(
                bam,
                &out_bam,
                &flagstat_before,
                &flagstat_after,
                &idxstats_before,
                &idxstats_after,
                &summary,
                params,
            ),
        };
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_core::ArtifactRef {
                    name: "bam".to_string(),
                    path: bam.to_path_buf(),
                }],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "optical_duplicates": params.optical_duplicates,
                "umi_policy": params.umi_policy,
                "duplicate_action": params.duplicate_action,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &[
                "markdup_bam",
                "markdup_bai",
                "flagstat_before",
                "flagstat_after",
                "idxstats_before",
                "idxstats_after",
                "summary",
                "stage_metrics",
            ],
        )
    }
}

pub mod complexity {
    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::ComplexityEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Complexity.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &ComplexityEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Complexity, out_dir);
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: tool.command.clone(),
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_core::ArtifactRef {
                    name: "bam".to_string(),
                    path: bam.to_path_buf(),
                }],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "min_reads": params.min_reads,
                "projection_points": params.projection_points,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["complexity_report", "preseq", "summary"],
        )
    }
}

pub mod coverage {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::CoverageEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Coverage.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &CoverageEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let mut outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Coverage, out_dir);
        let prefix = out_dir.join("coverage");
        let depth_path = out_dir.join("coverage.depth.txt");
        let summary_path = out_dir.join("coverage.mosdepth.summary.txt");
        let command = match tool.tool_id.0.as_str() {
            "samtools" => {
                outputs.push(bijux_core::ArtifactRef {
                    name: "coverage_depth".to_string(),
                    path: depth_path.clone(),
                });
                crate::tools::samtools::depth_args(bam, &depth_path, &summary_path)
            }
            _ => crate::tools::mosdepth::args(bam, &prefix, params),
        };
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_core::ArtifactRef {
                    name: "bam".to_string(),
                    path: bam.to_path_buf(),
                }],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "regions": params.regions,
                "depth_thresholds": params.depth_thresholds,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(plan, &["coverage_summary", "stage_metrics"])
    }
}

pub mod recalibration {
    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::BqsrEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Recalibration.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &BqsrEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::stages_support::audit_outputs(
            bijux_domain_bam::BamStage::Recalibration,
            out_dir,
        );
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: tool.command.clone(),
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_core::ArtifactRef {
                    name: "bam".to_string(),
                    path: bam.to_path_buf(),
                }],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "known_sites": params.known_sites,
                "mode": params.mode,
                "skip_criteria": params.skip_criteria,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["recal_bam", "recal_bai", "recal_report", "summary"],
        )
    }
}
