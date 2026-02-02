use std::path::Path;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId(tool.to_string()),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn output_path(plan: &bijux_core::StagePlanV1, name: &str) -> std::path::PathBuf {
    plan.io
        .outputs
        .iter()
        .find(|output| output.name == name)
        .map_or_else(
            || panic!("missing output {name} for stage {}", plan.stage_id.0),
            |output| output.path.clone(),
        )
}

#[test]
fn bam_plan_integration_has_stable_stage_chain() -> Result<()> {
    let out = Path::new("out");
    let input = Path::new("reads.bam");

    let validate = bijux_stages_bam::bam::validate::plan(
        &dummy_tool("samtools"),
        input,
        None,
        None,
        out.join("validate").as_path(),
    )?;
    assert_eq!(validate.stage_id.0, "bam.validate");

    let qc_pre = bijux_stages_bam::bam::qc_pre::plan(
        &dummy_tool("samtools"),
        input,
        out.join("qc_pre").as_path(),
    )?;
    assert_eq!(qc_pre.stage_id.0, "bam.qc_pre");

    let filter_params = bijux_domain_bam::FilterEffectiveParams {
        mapq_threshold: 30,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 30,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let filter = bijux_stages_bam::bam::filter::plan(
        &dummy_tool("samtools"),
        input,
        out.join("filter").as_path(),
        &filter_params,
    )?;
    assert_eq!(filter.stage_id.0, "bam.filter");

    let markdup_params = bijux_domain_bam::MarkDupEffectiveParams {
        optical_duplicates: bijux_domain_bam::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_domain_bam::UmiPolicy::Ignore,
        duplicate_action: bijux_domain_bam::DuplicateAction::Mark,
    };
    let markdup = bijux_stages_bam::bam::markdup::plan(
        &dummy_tool("gatk"),
        output_path(&filter, "filtered_bam").as_path(),
        out.join("markdup").as_path(),
        &markdup_params,
    )?;
    assert_eq!(markdup.stage_id.0, "bam.markdup");

    let coverage_params = bijux_domain_bam::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
    };
    let coverage = bijux_stages_bam::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        output_path(&markdup, "markdup_bam").as_path(),
        out.join("coverage").as_path(),
        &coverage_params,
    )?;
    assert_eq!(coverage.stage_id.0, "bam.coverage");

    let damage_params = bijux_domain_bam::DamageEffectiveParams {
        udg_model: bijux_domain_bam::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
    };
    let damage = bijux_stages_bam::bam::damage::plan(
        &dummy_tool("pydamage"),
        output_path(&markdup, "markdup_bam").as_path(),
        out.join("damage").as_path(),
        &damage_params,
    )?;
    assert_eq!(damage.stage_id.0, "bam.damage");

    for plan in [&validate, &qc_pre, &filter, &markdup, &coverage, &damage] {
        assert!(
            !plan.io.outputs.is_empty(),
            "stage {} missing outputs",
            plan.stage_id.0
        );
    }

    Ok(())
}
