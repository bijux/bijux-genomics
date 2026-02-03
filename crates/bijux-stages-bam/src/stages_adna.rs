pub mod damage {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::DamageEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Damage.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &DamageEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Damage, out_dir);
        let out_json = out_dir.join("damage.pydamage.json");
        let command = match tool.tool_id.0.as_str() {
            "mapdamage2" => crate::tools::mapdamage2::damage_args(bam, out_dir, params),
            _ => crate::tools::pydamage::args(bam, &out_json, params),
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
                "udg_model": params.udg_model,
                "pmd_threshold_5p": params.pmd_threshold_5p,
                "pmd_threshold_3p": params.pmd_threshold_3p,
                "trim_5p": params.trim_5p,
                "trim_3p": params.trim_3p,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["damage_pydamage", "damage_mapdamage2", "stage_metrics"],
        )
    }
}

pub mod authenticity {
    //! Plan for bam.authenticity stage.

    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::AuthenticityEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Authenticity.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &AuthenticityEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Authenticity, out_dir);
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
                "mode": params.mode,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["authenticity_report", "summary", "stage_metrics"],
        )
    }
}

pub mod contamination {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::ContaminationEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Contamination.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &ContaminationEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::stages_support::audit_outputs(
            bijux_domain_bam::BamStage::Contamination,
            out_dir,
        );
        let report = out_dir.join("contamination.json");
        let summary = out_dir.join("contamination.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: crate::tools::authenticct::args_with_outputs(
                    bam, &report, &summary, params,
                ),
            },
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
                "reference_panels": params.reference_panels,
                "scope": params.scope,
                "prior": params.prior,
                "sex_specific": params.sex_specific,
                "assumptions": params.assumptions,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["contamination_report", "summary", "stage_metrics"],
        )
    }
}

pub mod sex {
    use std::path::Path;

    use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
    use bijux_domain_bam::params::SexEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Sex.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &SexEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Sex, out_dir);
        let report = out_dir.join("sex.json");
        let summary = out_dir.join("sex.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: bijux_core::CommandSpecV1 {
                template: crate::tools::rxy::args_with_outputs(bam, &report, &summary, params),
            },
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
                "expected_sex": params.expected_sex,
                "method": params.method,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["sex_report", "summary", "stage_metrics"],
        )
    }
}
