pub mod haplogroups {
    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::HaplogroupEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Haplogroups.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &HaplogroupEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Haplogroups, out_dir);
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
                "reference_panel": params.reference_panel,
                "min_coverage": params.min_coverage,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["haplogroups", "summary", "stage_metrics"],
        )
    }
}

pub mod genotyping {
    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::GenotypingEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Genotyping.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &GenotypingEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Genotyping, out_dir);
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
                "caller": params.caller,
                "min_posterior": params.min_posterior,
                "min_call_rate": params.min_call_rate,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["genotyping_report", "summary", "stage_metrics"],
        )
    }
}

pub mod kinship {
    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::KinshipEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Kinship.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &KinshipEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Kinship, out_dir);
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
                "reference_panel": params.reference_panel,
                "min_overlap_snps": params.min_overlap_snps,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["kinship_report", "summary", "stage_metrics"],
        )
    }
}

pub mod bias_mitigation {
    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::BiasMitigationEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::BiasMitigation.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &BiasMitigationEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::stages_support::audit_outputs(
            bijux_domain_bam::BamStage::BiasMitigation,
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
                "gc_bias_correction": params.gc_bias_correction,
                "map_bias_correction": params.map_bias_correction,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["bias_report", "summary", "stage_metrics"],
        )
    }
}
