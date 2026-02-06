use std::fs;
use std::path::Path;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, StagePlanJsonV1 as StagePlanJson, ToolConstraints,
    ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
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

fn snapshot_path(name: &str) -> Result<std::path::PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    Ok(Path::new(&manifest_dir)
        .join("tests")
        .join("snapshots")
        .join(name))
}

fn assert_snapshot(name: &str, payload: &serde_json::Value) -> Result<()> {
    let rendered = serde_json::to_string_pretty(payload)?;
    let path = snapshot_path(name)?;
    if std::env::var("UPDATE_SNAPSHOTS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        bijux_infra::write_bytes(path, rendered)?;
        return Ok(());
    }
    let snapshot = fs::read_to_string(path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn bam_pipeline_plan_snapshot_is_stable() -> Result<()> {
    let bam = Path::new("reads.bam");
    let out = Path::new("out");
    let validate = bijux_planner_bam::tool_adapters::bam::validate::plan(
        &dummy_tool("samtools"),
        bam,
        None,
        None,
        out.join("validate").as_path(),
    )?;
    let qc_pre = bijux_planner_bam::tool_adapters::bam::qc_pre::plan(
        &dummy_tool("samtools"),
        bam,
        out.join("qc_pre").as_path(),
    )?;
    let filter_params = bijux_domain_bam::params::FilterEffectiveParams {
        mapq_threshold: 30,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 30,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let filter = bijux_planner_bam::tool_adapters::bam::filter::plan(
        &dummy_tool("samtools"),
        bam,
        out.join("filter").as_path(),
        &filter_params,
    )?;
    let markdup_params = bijux_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_domain_bam::params::DuplicateAction::Mark,
    };
    let markdup = bijux_planner_bam::tool_adapters::bam::markdup::plan(
        &dummy_tool("gatk"),
        bam,
        out.join("markdup").as_path(),
        &markdup_params,
    )?;
    let coverage_params = bijux_domain_bam::params::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
    };
    let coverage = bijux_planner_bam::tool_adapters::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        bam,
        out.join("coverage").as_path(),
        &coverage_params,
    )?;
    let damage_params = bijux_domain_bam::params::DamageEffectiveParams {
        udg_model: bijux_domain_bam::params::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
    };
    let damage = bijux_planner_bam::tool_adapters::bam::damage::plan(
        &dummy_tool("pydamage"),
        bam,
        out.join("damage").as_path(),
        &damage_params,
    )?;
    let sex_params = bijux_domain_bam::params::SexEffectiveParams {
        expected_sex: None,
        method: "rxy".to_string(),
    };
    let sex = bijux_planner_bam::tool_adapters::bam::sex::plan(
        &dummy_tool("rxy"),
        bam,
        out.join("sex").as_path(),
        &sex_params,
    )?;
    let contamination_params = bijux_domain_bam::params::ContaminationEffectiveParams {
        reference_panels: Vec::new(),
        scope: bijux_domain_bam::params::ContaminationScope::Both,
        prior: None,
        sex_specific: false,
        assumptions: None,
    };
    let contamination = bijux_planner_bam::tool_adapters::bam::contamination::plan(
        &dummy_tool("authenticct"),
        bam,
        out.join("contam").as_path(),
        &contamination_params,
    )?;

    let plans = [
        validate,
        qc_pre,
        filter,
        markdup,
        coverage,
        damage,
        sex,
        contamination,
    ];
    let payload = serde_json::json!({
        "stages": plans
            .iter()
            .map(StagePlanJson::from_plan)
            .collect::<Vec<_>>()
    });
    assert_snapshot("pipeline__bam__bam-to-bam__adna_shotgun__v1.json", &payload)
}
