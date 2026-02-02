use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
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

#[test]
#[allow(clippy::too_many_lines)]
fn bam_plan_integration_is_consistent() -> Result<()> {
    let validate = bijux_stages_bam::bam::validate::plan(
        &dummy_tool("samtools"),
        Path::new("reads.bam"),
        None,
        None,
        Path::new("out/validate"),
    )?;

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
        Path::new("reads.bam"),
        Path::new("out/filter"),
        &filter_params,
    )?;

    let markdup_params = bijux_domain_bam::MarkDupEffectiveParams {
        optical_duplicates: bijux_domain_bam::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_domain_bam::UmiPolicy::Ignore,
        duplicate_action: bijux_domain_bam::DuplicateAction::Mark,
    };
    let markdup = bijux_stages_bam::bam::markdup::plan(
        &dummy_tool("gatk"),
        Path::new("out/filter/filtered.bam"),
        Path::new("out/markdup"),
        &markdup_params,
    )?;

    let coverage_params = bijux_domain_bam::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
    };
    let coverage = bijux_stages_bam::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        Path::new("out/markdup/markdup.bam"),
        Path::new("out/coverage"),
        &coverage_params,
    )?;

    let damage_params = bijux_domain_bam::DamageEffectiveParams {
        udg_model: bijux_domain_bam::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
    };
    let damage = bijux_stages_bam::bam::damage::plan(
        &dummy_tool("pydamage"),
        Path::new("out/markdup/markdup.bam"),
        Path::new("out/damage"),
        &damage_params,
    )?;

    let sex_params = bijux_domain_bam::SexEffectiveParams {
        expected_sex: None,
        method: "rxy".to_string(),
    };
    let sex = bijux_stages_bam::bam::sex::plan(
        &dummy_tool("rxy"),
        Path::new("out/markdup/markdup.bam"),
        Path::new("out/sex"),
        &sex_params,
    )?;

    let stage_ids = vec![
        validate.stage_id.clone(),
        filter.stage_id.clone(),
        markdup.stage_id.clone(),
        coverage.stage_id.clone(),
        damage.stage_id.clone(),
        sex.stage_id.clone(),
    ];
    let expected = vec![
        StageId("bam.validate".to_string()),
        StageId("bam.filter".to_string()),
        StageId("bam.markdup".to_string()),
        StageId("bam.coverage".to_string()),
        StageId("bam.damage".to_string()),
        StageId("bam.sex".to_string()),
    ];
    assert_eq!(stage_ids, expected);

    let filter_bam = filter
        .io
        .outputs
        .iter()
        .find(|output| output.name == "filtered_bam")
        .map(|output| output.path.clone())
        .ok_or_else(|| anyhow!("filtered_bam output missing"))?;
    let markdup_input = markdup
        .io
        .inputs
        .iter()
        .find(|input| input.name == "bam")
        .map(|input| input.path.clone())
        .ok_or_else(|| anyhow!("markdup input missing"))?;
    assert_eq!(filter_bam, markdup_input);

    let markdup_bam = markdup
        .io
        .outputs
        .iter()
        .find(|output| output.name == "markdup_bam")
        .map(|output| output.path.clone())
        .ok_or_else(|| anyhow!("markdup_bam output missing"))?;

    let coverage_input = coverage
        .io
        .inputs
        .iter()
        .find(|input| input.name == "bam")
        .map(|input| input.path.clone())
        .ok_or_else(|| anyhow!("coverage input missing"))?;
    assert_eq!(markdup_bam, coverage_input);

    let damage_input = damage
        .io
        .inputs
        .iter()
        .find(|input| input.name == "bam")
        .map(|input| input.path.clone())
        .ok_or_else(|| anyhow!("damage input missing"))?;
    assert_eq!(markdup_bam, damage_input);

    let sex_input = sex
        .io
        .inputs
        .iter()
        .find(|input| input.name == "bam")
        .map(|input| input.path.clone())
        .ok_or_else(|| anyhow!("sex input missing"))?;
    assert_eq!(markdup_bam, sex_input);

    Ok(())
}
