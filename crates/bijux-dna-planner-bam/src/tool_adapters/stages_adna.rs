//! Stage adapters for aDNA-specific downstream stages.

pub mod damage {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::DamageEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Damage.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &DamageEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Damage,
            out_dir,
        );
        let out_json = out_dir.join("damage.pydamage.json");
        let command = match tool.tool_id.as_str() {
            "mapdamage2" => {
                crate::tool_adapters::tools::mapdamage2::damage_args(bam, out_dir, params)
            }
            "damageprofiler" => crate::tool_adapters::tools::damageprofiler::args(
                bam,
                &out_dir.join("damage.profiler.json"),
                params,
            ),
            "addeam" => crate::tool_adapters::tools::addeam::args(
                bam,
                &out_dir.join("damage.addeam.json"),
                params,
            ),
            "ngsbriggs" => crate::tool_adapters::tools::ngsbriggs::args(
                bam,
                &out_dir.join("damage.ngsbriggs.json"),
                params,
            ),
            _ => crate::tool_adapters::tools::pydamage::args(bam, &out_json, params),
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("bam"),
                    bam.to_path_buf(),
                    ArtifactRole::Bam,
                )],
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
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["damage_pydamage", "damage_mapdamage2", "stage_metrics"],
        )
    }
}

pub mod authenticity {
    //! Plan for bam.authenticity stage.

    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::AuthenticityEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Authenticity.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &AuthenticityEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Authenticity,
            out_dir,
        );
        let report = out_dir.join("authenticity.json");
        let summary = out_dir.join("authenticity.summary.json");
        let mut outputs = outputs;
        if tool.tool_id.as_str() == "pmdtools" {
            outputs.push(bijux_dna_stage_contract::ArtifactRef::optional(
                ArtifactId::from_static("pmd_filtered_bam"),
                out_dir.join("pmd.filtered.bam"),
                ArtifactRole::Bam,
            ));
        }
        let command = match tool.tool_id.as_str() {
            "pmdtools" => crate::tool_adapters::tools::pmdtools::filter_args(
                bam,
                &out_dir.join("pmd.filtered.bam"),
                &report,
                &summary,
                params,
            ),
            _ => crate::tool_adapters::tools::authenticity_signal::args_with_outputs(
                bam, &report, &summary, params,
            ),
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("bam"),
                    bam.to_path_buf(),
                    ArtifactRole::Bam,
                )],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "mode": params.mode,
                "pmd_filter_enabled": tool.tool_id.as_str() == "pmdtools",
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["authenticity_report", "summary", "stage_metrics"],
        )
    }
}

pub mod contamination {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::ContaminationEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Contamination.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &ContaminationEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Contamination,
            out_dir,
        );
        let report = out_dir.join("contamination.json");
        let summary = out_dir.join("contamination.summary.json");
        let command = match tool.tool_id.as_str() {
            "schmutzi" => crate::tool_adapters::tools::schmutzi::args_with_outputs(
                bam, &report, &summary, params,
            ),
            "verifybamid2" => crate::tool_adapters::tools::verifybamid2::args_with_outputs(
                bam, &report, &summary, params,
            ),
            "contammix" => crate::tool_adapters::tools::contammix::args_with_outputs(
                bam, &report, &summary, params,
            ),
            _ => crate::tool_adapters::tools::authenticity::args_with_outputs(
                bam, &report, &summary, params,
            ),
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("bam"),
                    bam.to_path_buf(),
                    ArtifactRole::Bam,
                )],
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
                "tool_scope": match tool.tool_id.as_str() {
                    "schmutzi" => "mt",
                    "verifybamid2" | "contammix" => "nuclear",
                    _ => "both",
                },
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["contamination_report", "summary", "stage_metrics"],
        )
    }
}

pub mod sex {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::SexEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Sex.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &SexEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Sex,
            out_dir,
        );
        let report = out_dir.join("sex.json");
        let summary = out_dir.join("sex.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: match (tool.tool_id.as_str(), params.method.as_str()) {
                    ("angsd", _) | (_, "angsd") => {
                        crate::tool_adapters::tools::angsd_sex::args_with_outputs(
                            bam, &report, &summary, params,
                        )
                    }
                    _ => crate::tool_adapters::tools::rxy::args_with_outputs(
                        bam, &report, &summary, params,
                    ),
                },
            },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("bam"),
                    bam.to_path_buf(),
                    ArtifactRole::Bam,
                )],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "expected_sex": params.expected_sex,
                "method": params.method,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["sex_report", "summary", "stage_metrics"],
        )
    }
}
