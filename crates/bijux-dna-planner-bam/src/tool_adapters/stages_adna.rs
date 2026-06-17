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

    fn retained_output_ids(tool_id: &str) -> &'static [&'static str] {
        match tool_id {
            "addeam" => &[
                "damage_report",
                "terminal_position_metrics",
                "parser_output",
                "stage_metrics",
                "damage_profile",
                "damage_clusters",
            ],
            "damageprofiler" | "mapdamage2" => &[
                "damage_report",
                "terminal_position_metrics",
                "parser_output",
                "stage_metrics",
                "damage_profile",
                "damage_plot",
            ],
            "ngsbriggs" => &[
                "damage_report",
                "terminal_position_metrics",
                "parser_output",
                "stage_metrics",
                "damage_profile",
                "damage_parameters",
            ],
            "pmdtools" => &[
                "damage_report",
                "terminal_position_metrics",
                "parser_output",
                "stage_metrics",
                "pmd_scores",
                "damage_profile",
            ],
            _ => &[
                "damage_report",
                "terminal_position_metrics",
                "parser_output",
                "stage_metrics",
                "damage_profile",
            ],
        }
    }

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &DamageEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let required_outputs = retained_output_ids(tool.tool_id.as_str());
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Damage,
            out_dir,
        )
        .into_iter()
        .filter(|artifact| required_outputs.contains(&artifact.name.as_str()))
        .collect();
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
            "pmdtools" => crate::tool_adapters::tools::pmdtools::damage_args(
                bam,
                &out_dir.join("damage.pmdtools.json"),
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
                "damage_tool_profile": params.damage_tool_profile,
                "evidence_only": params.evidence_only,
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
        crate::tool_adapters::stages_support::ensure_required_outputs(plan, required_outputs)
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

    fn retained_output_ids(tool_id: &str) -> &'static [&'static str] {
        match tool_id {
            "damageprofiler" => &[
                "authenticity_report",
                "summary",
                "stage_metrics",
                "damage_profile",
                "damage_plot",
            ],
            "pmdtools" => {
                &["authenticity_report", "summary", "stage_metrics", "pmd_scores", "damage_profile"]
            }
            _ => &["authenticity_report", "summary", "stage_metrics"],
        }
    }

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &AuthenticityEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let required_outputs = retained_output_ids(tool.tool_id.as_str());
        let outputs: Vec<_> = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Authenticity,
            out_dir,
        )
        .into_iter()
        .filter(|artifact| required_outputs.contains(&artifact.name.as_str()))
        .collect();
        let report = out_dir.join("authenticity.json");
        let summary = out_dir.join("authenticity.summary.json");
        let command = match tool.tool_id.as_str() {
            "pmdtools" => {
                crate::tool_adapters::tools::pmdtools::filter_args(bam, &report, &summary, params)
            }
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
                "evidence_only": params.evidence_only,
                "disallow_certification": params.disallow_certification,
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
        crate::tool_adapters::stages_support::ensure_required_outputs(plan, required_outputs)
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
        bam_index: Option<&Path>,
        reference: Option<&Path>,
        out_dir: &Path,
        params: &ContaminationEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let mut outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Contamination,
            out_dir,
        );
        let report = out_dir.join("contamination.json");
        let summary = out_dir.join("contamination.summary.json");
        let contamination_estimate = out_dir.join("contamination.estimate.json");
        let contammix_report = out_dir.join("contammix.report.txt");
        let mt_consensus = out_dir.join("mt_consensus.fasta");
        outputs.push(bijux_dna_stage_contract::ArtifactRef::required(
            ArtifactId::from_static("contamination_estimate"),
            contamination_estimate.clone(),
            ArtifactRole::ReportJson,
        ));
        match tool.tool_id.as_str() {
            "contammix" => outputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("contammix_report"),
                contammix_report.clone(),
                ArtifactRole::Report,
            )),
            "schmutzi" => outputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("mt_consensus"),
                mt_consensus.clone(),
                ArtifactRole::Reference,
            )),
            _ => {}
        }
        let panel_paths = params.reference_panels.iter().map(Path::new).collect::<Vec<_>>();
        let command = match tool.tool_id.as_str() {
            "schmutzi" => crate::tool_adapters::tools::schmutzi::args_with_outputs(
                bam, reference, &report, &summary, params,
            ),
            "verifybamid2" => crate::tool_adapters::tools::verifybamid2::args_with_outputs(
                bam,
                bam_index,
                reference,
                &panel_paths,
                &report,
                &summary,
                params,
            ),
            "contammix" => crate::tool_adapters::tools::contammix::args_with_outputs(
                bam,
                bam_index,
                reference,
                &panel_paths,
                &report,
                &summary,
                params,
            ),
            _ => crate::tool_adapters::tools::authenticity::args_with_outputs(
                bam, &report, &summary, params,
            ),
        };
        let mut inputs = vec![bijux_dna_stage_contract::ArtifactRef::required(
            ArtifactId::from_static("bam"),
            bam.to_path_buf(),
            ArtifactRole::Bam,
        )];
        if let Some(bam_index) = bam_index {
            inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("bam_bai"),
                bam_index.to_path_buf(),
                ArtifactRole::Index,
            ));
        }
        if let Some(reference) = reference {
            inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("reference"),
                reference.to_path_buf(),
                ArtifactRole::Reference,
            ));
        }
        for (idx, panel) in panel_paths.iter().enumerate() {
            let artifact_id = if idx == 0 {
                ArtifactId::from_static("reference_panel")
            } else {
                ArtifactId::new(format!("reference_panel_{}", idx + 1))
            };
            inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                artifact_id,
                panel.to_path_buf(),
                ArtifactRole::Reference,
            ));
        }
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO { inputs, outputs },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "bai": bam_index,
                "reference": reference,
                "reference_panels": params.reference_panels,
                "scope": params.scope,
                "prior": params.prior,
                "sex_specific": params.sex_specific,
                "assumptions": params.assumptions,
                "required_reference_digest": params.required_reference_digest,
                "chromosome_system": params.chromosome_system,
                "minimum_mean_coverage": params.minimum_mean_coverage,
                "emit_confidence_caveats": params.emit_confidence_caveats,
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
            &["contamination_report", "summary", "stage_metrics", "contamination_estimate"],
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

    fn retained_output_ids(tool_id: &str) -> &'static [&'static str] {
        match tool_id {
            "angsd" => &["sex_report", "summary", "stage_metrics", "population_metrics"],
            "yleaf" => &["sex_report", "summary", "stage_metrics", "haplogroup_report"],
            _ => &["sex_report", "summary", "stage_metrics", "sex_estimate"],
        }
    }

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &SexEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let required_outputs = retained_output_ids(tool.tool_id.as_str());
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Sex,
            out_dir,
        )
        .into_iter()
        .filter(|artifact| required_outputs.contains(&artifact.name.as_str()))
        .collect();
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
                    ("yleaf", _) | (_, "yleaf") => {
                        crate::tool_adapters::tools::yleaf_sex::args_with_outputs(
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
                "chromosome_system": params.chromosome_system,
                "minimum_y_sites": params.minimum_y_sites,
                "refuse_without_context": params.refuse_without_context,
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
        crate::tool_adapters::stages_support::ensure_required_outputs(plan, required_outputs)
    }
}
