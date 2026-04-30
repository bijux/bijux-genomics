//! Stage adapters for downstream interpretation and report stages.

pub mod haplogroups {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::HaplogroupEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Haplogroups.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &HaplogroupEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Haplogroups,
            out_dir,
        );
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: bijux_dna_core::prelude::CommandSpecV1 {
                template: tool.command.template.to_vec(),
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
                "reference_panel": params.reference_panel,
                "reference_build": params.reference_build,
                "min_coverage": params.min_coverage,
                "population_scope": params.population_scope,
                "refuse_without_population_context": params.refuse_without_population_context,
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
            &["haplogroups", "summary", "stage_metrics"],
        )
    }
}

pub mod genotyping {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::GenotypingEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Genotyping.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &GenotypingEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let mut outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Genotyping,
            out_dir,
        );
        let vcf_gz = out_dir.join("genotyping.vcf.gz");
        let tbi = out_dir.join("genotyping.vcf.gz.tbi");
        let gl_json = out_dir.join("genotyping.gl.json");
        outputs.push(bijux_dna_stage_contract::ArtifactRef::optional(
            ArtifactId::from_static("genotyping_vcf"),
            vcf_gz.clone(),
            ArtifactRole::ReportJson,
        ));
        outputs.push(bijux_dna_stage_contract::ArtifactRef::optional(
            ArtifactId::from_static("genotyping_vcf_tbi"),
            tbi.clone(),
            ArtifactRole::Index,
        ));
        outputs.push(bijux_dna_stage_contract::ArtifactRef::optional(
            ArtifactId::from_static("genotyping_gl"),
            gl_json.clone(),
            ArtifactRole::ReportJson,
        ));
        let report = out_dir.join("genotyping.json");
        let summary = out_dir.join("genotyping.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: crate::tool_adapters::tools::genotyping::args_with_outputs(
                    tool.tool_id.as_str(),
                    bam,
                    crate::tool_adapters::tools::genotyping::GenotypingOutputs {
                        report: &report,
                        summary: &summary,
                        vcf_gz: &vcf_gz,
                        tbi: &tbi,
                        gl_json: &gl_json,
                    },
                    params,
                ),
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
                "caller": params.caller,
                "min_posterior": params.min_posterior,
                "min_call_rate": params.min_call_rate,
                "producer_contract": {
                    "vcf": vcf_gz,
                    "tbi": tbi,
                    "gl": gl_json,
                }
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
            &["genotyping_report", "summary", "stage_metrics"],
        )
    }
}

pub mod kinship {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::KinshipEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Kinship.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &KinshipEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        if params.reference_panel.trim().is_empty() {
            return Err(anyhow::anyhow!("bam.kinship requires non-empty reference_panel"));
        }
        if params.min_overlap_snps == 0 {
            return Err(anyhow::anyhow!("bam.kinship requires min_overlap_snps > 0"));
        }
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Kinship,
            out_dir,
        );
        let report = out_dir.join("kinship.json");
        let summary = out_dir.join("kinship.summary.json");
        let segments = out_dir.join("kinship.segments.tsv");
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: crate::tool_adapters::tools::kinship::args_with_outputs(
                    tool.tool_id.as_str(),
                    bam,
                    &report,
                    &summary,
                    &segments,
                    params,
                ),
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
                "reference_panel": params.reference_panel,
                "reference_build": params.reference_build,
                "population_scope": params.population_scope,
                "min_overlap_snps": params.min_overlap_snps,
                "requires_cohort_context": params.requires_cohort_context,
                "pseudo_haploid_policy": "refuse_unless_explicit_conversion",
                "required_inputs": ["diploid_genotypes_or_explicit_pseudohap_conversion"],
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
            &["kinship_report", "summary", "stage_metrics"],
        )
    }
}

pub mod bias_mitigation {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::BiasMitigationEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::BiasMitigation.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &BiasMitigationEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::BiasMitigation,
            out_dir,
        );
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: bijux_dna_core::prelude::CommandSpecV1 {
                template: tool.command.template.to_vec(),
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
                "gc_bias_correction": params.gc_bias_correction,
                "map_bias_correction": params.map_bias_correction,
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
            &["bias_report", "summary", "stage_metrics"],
        )
    }
}
