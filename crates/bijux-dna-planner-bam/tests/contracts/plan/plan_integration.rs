use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

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

fn output_path(plan: &bijux_dna_stage_contract::StagePlanV1, name: &str) -> std::path::PathBuf {
    plan.io.outputs.iter().find(|output| output.name.as_str() == name).map_or_else(
        || panic!("missing output {name} for stage {}", plan.stage_id.0),
        |output| output.path.clone(),
    )
}

#[test]
fn bam_plan_integration_has_stable_stage_chain() -> Result<()> {
    let out = Path::new("out");
    let input = Path::new("reads.bam");

    let validate = bijux_dna_planner_bam::tool_adapters::bam::validate::plan(
        &dummy_tool("samtools"),
        input,
        None,
        None,
        out.join("validate").as_path(),
    )?;
    assert_eq!(validate.stage_id.0, "bam.validate");

    let qc_pre = bijux_dna_planner_bam::tool_adapters::bam::qc_pre::plan(
        &dummy_tool("samtools"),
        input,
        out.join("qc_pre").as_path(),
    )?;
    assert_eq!(qc_pre.stage_id.0, "bam.qc_pre");

    let filter_params = bijux_dna_domain_bam::params::FilterEffectiveParams {
        mapq_threshold: 30,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 30,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let filter = bijux_dna_planner_bam::tool_adapters::bam::filter::plan(
        &dummy_tool("samtools"),
        input,
        out.join("filter").as_path(),
        &filter_params,
    )?;
    assert_eq!(filter.stage_id.0, "bam.filter");

    let markdup_params = bijux_dna_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_dna_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_dna_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_dna_domain_bam::params::DuplicateAction::Mark,
    };
    let markdup = bijux_dna_planner_bam::tool_adapters::bam::markdup::plan(
        &dummy_tool("gatk"),
        output_path(&filter, "filtered_bam").as_path(),
        out.join("markdup").as_path(),
        &markdup_params,
    )?;
    assert_eq!(markdup.stage_id.0, "bam.markdup");

    let coverage_params = bijux_dna_domain_bam::params::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let coverage = bijux_dna_planner_bam::tool_adapters::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        output_path(&markdup, "markdup_bam").as_path(),
        out.join("coverage").as_path(),
        &coverage_params,
    )?;
    assert_eq!(coverage.stage_id.0, "bam.coverage");

    let damage_params = bijux_dna_domain_bam::params::DamageEffectiveParams {
        udg_model: bijux_dna_domain_bam::params::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
        damage_tool_profile: Some("ancient".to_string()),
        evidence_only: true,
    };
    let damage = bijux_dna_planner_bam::tool_adapters::bam::damage::plan(
        &dummy_tool("pydamage"),
        output_path(&markdup, "markdup_bam").as_path(),
        out.join("damage").as_path(),
        &damage_params,
    )?;
    assert_eq!(damage.stage_id.0, "bam.damage");

    for plan in [&validate, &qc_pre, &filter, &markdup, &coverage, &damage] {
        assert!(!plan.io.outputs.is_empty(), "stage {} missing outputs", plan.stage_id.0);
    }

    Ok(())
}

#[test]
fn mini_local_chain_align_markdup_damage_coverage_has_expected_artifacts() -> Result<()> {
    let out = Path::new("out-mini");
    let r1 = Path::new("sample_R1.fastq.gz");
    let r2 = Path::new("sample_R2.fastq.gz");
    let reference = Path::new("reference.fa");

    let align_params = bijux_dna_domain_bam::params::AlignEffectiveParams {
        aligner: "bwa".to_string(),
        strategy_id: "bwa_mem_default".to_string(),
        preset: "default".to_string(),
        mode: "end_to_end".to_string(),
        threads: 1,
        reference: reference.display().to_string(),
        reference_digest: "sha256:dummy".to_string(),
        rg_policy: bijux_dna_domain_bam::types::ReadGroupPolicy::Regenerate,
        read_group: bijux_dna_domain_bam::params::ReadGroupSpec::with_defaults("sample"),
        sensitivity_profile: Some("default".to_string()),
        seed_length: Some(19),
        build_indices: true,
        emit_stats: true,
    };
    let align = bijux_dna_planner_bam::tool_adapters::bam::align::plan(
        &dummy_tool("bwa"),
        r1,
        Some(r2),
        reference,
        "sample",
        &align_params,
        out.join("align").as_path(),
    )?;
    assert_eq!(align.stage_id.0, "bam.align");
    assert!(align.io.outputs.iter().any(|o| o.name.as_str() == "align_bam"));

    let markdup_params = bijux_dna_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_dna_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_dna_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_dna_domain_bam::params::DuplicateAction::Mark,
    };
    let markdup = bijux_dna_planner_bam::tool_adapters::bam::markdup::plan(
        &dummy_tool("samtools"),
        output_path(&align, "align_bam").as_path(),
        out.join("markdup").as_path(),
        &markdup_params,
    )?;
    assert_eq!(markdup.stage_id.0, "bam.markdup");
    assert!(markdup.io.outputs.iter().any(|o| o.name.as_str() == "markdup_bam"));

    let damage_params = bijux_dna_domain_bam::params::DamageEffectiveParams {
        udg_model: bijux_dna_domain_bam::params::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
        damage_tool_profile: Some("ancient".to_string()),
        evidence_only: true,
    };
    let damage = bijux_dna_planner_bam::tool_adapters::bam::damage::plan(
        &dummy_tool("damageprofiler"),
        output_path(&markdup, "markdup_bam").as_path(),
        out.join("damage").as_path(),
        &damage_params,
    )?;
    assert_eq!(damage.stage_id.0, "bam.damage");
    assert!(damage
        .io
        .outputs
        .iter()
        .any(|o| o.name.as_str() == "damage_pydamage" || o.name.as_str() == "damage_mapdamage2"));

    let coverage_params = bijux_dna_domain_bam::params::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 5, 10],
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let coverage = bijux_dna_planner_bam::tool_adapters::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        output_path(&markdup, "markdup_bam").as_path(),
        out.join("coverage").as_path(),
        &coverage_params,
    )?;
    assert_eq!(coverage.stage_id.0, "bam.coverage");
    assert!(coverage.io.outputs.iter().any(|o| o.name.as_str() == "coverage_summary"));
    Ok(())
}
