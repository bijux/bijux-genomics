use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
#[cfg(feature = "bam_downstream")]
use bijux_dna_domain_bam::params::{
    BiasMitigationEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams,
};
use bijux_dna_domain_bam::params::{
    ComplexityEffectiveParams, ContaminationEffectiveParams, CoverageEffectiveParams,
    DamageEffectiveParams, FilterEffectiveParams, MarkDupEffectiveParams, SexEffectiveParams,
    UdgModel,
};
use bijux_dna_domain_bam::required_audit_artifacts;
use bijux_dna_domain_bam::BamStage;

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: Vec::new() },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn assert_audit_outputs(stage: BamStage, plan: &bijux_dna_stage_contract::StagePlanV1) {
    let outputs: std::collections::HashSet<_> =
        plan.io.outputs.iter().map(|o| o.name.as_str()).collect();
    let spec = bijux_dna_domain_bam::stage_spec(stage);
    for artifact in required_audit_artifacts(stage) {
        assert!(
            outputs.contains(artifact.name),
            "stage {} missing required output {}",
            stage.as_str(),
            artifact.name
        );
    }
    for required in spec.artifact_policy.required_outputs {
        assert!(
            outputs.contains(*required),
            "stage {} missing required output {}",
            stage.as_str(),
            required
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn bam_stage_artifacts_contract_is_complete() -> Result<()> {
    let bam = Path::new("reads.bam");
    let out = Path::new("out");

    let validate = bijux_dna_planner_bam::tool_adapters::bam::validate::plan(
        &dummy_tool("samtools"),
        bam,
        None,
        None,
        out,
    )?;
    assert_audit_outputs(BamStage::Validate, &validate);

    let qc_pre =
        bijux_dna_planner_bam::tool_adapters::bam::qc_pre::plan(&dummy_tool("samtools"), bam, out)?;
    assert_audit_outputs(BamStage::QcPre, &qc_pre);

    let filter_params = FilterEffectiveParams {
        mapq_threshold: 30,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 30,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let filter = bijux_dna_planner_bam::tool_adapters::bam::filter::plan(
        &dummy_tool("samtools"),
        bam,
        out,
        &filter_params,
    )?;
    assert_audit_outputs(BamStage::Filter, &filter);

    let mapq_filter = bijux_dna_planner_bam::tool_adapters::bam::mapq_filter::plan(
        &dummy_tool("samtools"),
        bam,
        out,
        &filter_params,
    )?;
    assert_audit_outputs(BamStage::MapqFilter, &mapq_filter);

    let length_filter = bijux_dna_planner_bam::tool_adapters::bam::length_filter::plan(
        &dummy_tool("samtools"),
        bam,
        out,
        &filter_params,
    )?;
    assert_audit_outputs(BamStage::LengthFilter, &length_filter);

    let overlap_correction = bijux_dna_planner_bam::tool_adapters::bam::overlap_correction::plan(
        &dummy_tool("samtools"),
        bam,
        out,
        &filter_params,
    )?;
    assert_audit_outputs(BamStage::OverlapCorrection, &overlap_correction);

    let mapping_summary = bijux_dna_planner_bam::tool_adapters::bam::mapping_summary::plan(
        &dummy_tool("samtools"),
        bam,
        out,
    )?;
    assert_audit_outputs(BamStage::MappingSummary, &mapping_summary);

    let markdup_params = MarkDupEffectiveParams {
        optical_duplicates: bijux_dna_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_dna_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_dna_domain_bam::params::DuplicateAction::Mark,
    };
    let markdup = bijux_dna_planner_bam::tool_adapters::bam::markdup::plan(
        &dummy_tool("gatk"),
        bam,
        out,
        &markdup_params,
    )?;
    assert_audit_outputs(BamStage::Markdup, &markdup);

    let complexity_params = ComplexityEffectiveParams {
        min_reads: 100_000,
        projection_points: vec![1_000_000, 2_000_000],
    };
    let complexity = bijux_dna_planner_bam::tool_adapters::bam::complexity::plan(
        &dummy_tool("preseq"),
        bam,
        out,
        &complexity_params,
    )?;
    assert_audit_outputs(BamStage::Complexity, &complexity);

    let coverage_params = CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let coverage = bijux_dna_planner_bam::tool_adapters::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        bam,
        out,
        &coverage_params,
    )?;
    assert_audit_outputs(BamStage::Coverage, &coverage);

    let damage_params = DamageEffectiveParams {
        udg_model: UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
        damage_tool_profile: Some("ancient".to_string()),
        evidence_only: true,
    };
    let damage = bijux_dna_planner_bam::tool_adapters::bam::damage::plan(
        &dummy_tool("pydamage"),
        bam,
        out,
        &damage_params,
    )?;
    assert_audit_outputs(BamStage::Damage, &damage);

    let authenticity_params = bijux_dna_domain_bam::params::AuthenticityEffectiveParams {
        mode: "aggregate".to_string(),
        evidence_only: true,
        disallow_certification: true,
    };
    let authenticity = bijux_dna_planner_bam::tool_adapters::bam::authenticity::plan(
        &dummy_tool("auth"),
        bam,
        out,
        &authenticity_params,
    )?;
    assert_audit_outputs(BamStage::Authenticity, &authenticity);

    let contamination_params = ContaminationEffectiveParams {
        reference_panels: vec!["panel.vcf".to_string()],
        scope: bijux_dna_domain_bam::params::ContaminationScope::Both,
        prior: None,
        sex_specific: false,
        assumptions: None,
        required_reference_digest: Some("sha256:panel".to_string()),
        chromosome_system: Some("xy".to_string()),
        minimum_mean_coverage: Some(0.75),
        emit_confidence_caveats: true,
    };
    let contamination = bijux_dna_planner_bam::tool_adapters::bam::contamination::plan(
        &dummy_tool("authentic"),
        bam,
        out,
        &contamination_params,
    )?;
    assert_audit_outputs(BamStage::Contamination, &contamination);

    let sex_params = SexEffectiveParams {
        expected_sex: None,
        method: "rxy".to_string(),
        chromosome_system: Some("xy".to_string()),
        minimum_y_sites: Some(100),
        refuse_without_context: true,
    };
    let sex = bijux_dna_planner_bam::tool_adapters::bam::sex::plan(
        &dummy_tool("rxy"),
        bam,
        out,
        &sex_params,
    )?;
    assert_audit_outputs(BamStage::Sex, &sex);

    #[cfg(feature = "bam_downstream")]
    {
        let bias_params =
            BiasMitigationEffectiveParams { gc_bias_correction: true, map_bias_correction: false };
        let bias = bijux_dna_planner_bam::tool_adapters::bam::bias_mitigation::plan(
            &dummy_tool("angsd"),
            bam,
            out,
            &bias_params,
        )?;
        assert_audit_outputs(BamStage::BiasMitigation, &bias);
    }

    let recal_params = bijux_dna_domain_bam::params::BqsrEffectiveParams {
        known_sites: vec!["known.vcf".to_string()],
        mode: bijux_dna_domain_bam::params::BqsrMode::Standard,
        skip_criteria: bijux_dna_domain_bam::params::RecalibrationSkipCriteria {
            min_mean_coverage: 2.0,
            min_breadth_1x: 0.5,
        },
    };
    let recal = bijux_dna_planner_bam::tool_adapters::bam::recalibration::plan(
        &dummy_tool("gatk"),
        bam,
        out,
        &recal_params,
    )?;
    assert_audit_outputs(BamStage::Recalibration, &recal);

    #[cfg(feature = "bam_downstream")]
    {
        let haplo_params = HaplogroupEffectiveParams {
            reference_panel: "rcrs.fasta".to_string(),
            reference_build: "GRCh38".to_string(),
            min_coverage: Some(5.0),
            population_scope: Some("ancient_eurasia".to_string()),
            refuse_without_population_context: true,
        };
        let haplogroups = bijux_dna_planner_bam::tool_adapters::bam::haplogroups::plan(
            &dummy_tool("yleaf"),
            bam,
            out,
            &haplo_params,
        )?;
        assert_audit_outputs(BamStage::Haplogroups, &haplogroups);

        let genotyping_params = GenotypingEffectiveParams {
            caller: "angsd".to_string(),
            min_posterior: Some(0.8),
            min_call_rate: Some(0.7),
        };
        let genotyping = bijux_dna_planner_bam::tool_adapters::bam::genotyping::plan(
            &dummy_tool("angsd"),
            bam,
            out,
            &genotyping_params,
        )?;
        assert_audit_outputs(BamStage::Genotyping, &genotyping);

        let kinship_params = KinshipEffectiveParams {
            reference_panel: "panel.vcf".to_string(),
            reference_build: "GRCh38".to_string(),
            population_scope: "ancient_eurasia".to_string(),
            min_overlap_snps: 200,
            requires_cohort_context: true,
        };
        let kinship = bijux_dna_planner_bam::tool_adapters::bam::kinship::plan(
            &dummy_tool("king"),
            bam,
            out,
            &kinship_params,
        )?;
        assert_audit_outputs(BamStage::Kinship, &kinship);
    }

    Ok(())
}

#[test]
fn reference_assets_use_fasta_fai_sidecar() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-reference-assets")?;
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&reference, b">chrM\nACGT\n")?;
    std::fs::write(temp.path().join("reference.fasta.fai"), b"chrM\t4\t6\t4\t5\n")?;
    std::fs::write(temp.path().join("reference.dict"), b"@SQ\tSN:chrM\tLN:4\n")?;

    let assets = bijux_dna_planner_bam::tool_adapters::stages_support::ensure_reference_assets(
        &reference, false, true,
    )?;
    assert!(assets.fai.ends_with("reference.fasta.fai"));
    assert!(assets.dict.ends_with("reference.dict"));
    Ok(())
}
