use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::{
    plan_bam_workflow_template_stage_plans, plan_stage, BamPipelineInputs, StagePlanRequest,
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

#[test]
fn contamination_requires_context_for_nuclear_tools() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-contamination-governance")?;
    let bam = temp.path().join("sample.bam");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, b"bam")?;
    std::fs::write(&reference, b">chr1\nACGT\n")?;

    let missing_context = plan_stage(StagePlanRequest {
        stage_id: BamStage::Contamination.as_str(),
        tool: &dummy_tool("verifybamid2"),
        out_dir: temp.path(),
        bam: Some(&bam),
        bam_index: None,
        r1: None,
        r2: None,
        reference: Some(&reference),
        sample_id: Some("sample"),
        params: Some(&serde_json::json!({
            "reference_panels": ["1kg-phase3"],
            "scope": "nuclear",
            "prior": 0.02,
            "sex_specific": false,
            "assumptions": "panel-based estimate"
        })),
    });
    assert!(missing_context.is_err());
    let message = format!("{}", missing_context.expect_err("missing context must fail"));
    assert!(message.contains("chromosome_system"));

    let plan = plan_stage(StagePlanRequest {
        stage_id: BamStage::Contamination.as_str(),
        tool: &dummy_tool("verifybamid2"),
        out_dir: temp.path(),
        bam: Some(&bam),
        bam_index: None,
        r1: None,
        r2: None,
        reference: Some(&reference),
        sample_id: Some("sample"),
        params: Some(&serde_json::json!({
            "reference_panels": ["1kg-phase3"],
            "scope": "nuclear",
            "prior": 0.02,
            "sex_specific": false,
            "assumptions": "panel-based estimate",
            "chromosome_system": "xy",
            "minimum_mean_coverage": 0.75,
            "emit_confidence_caveats": true
        })),
    })?;
    assert_eq!(plan.effective_params["chromosome_system"], "xy");
    assert_eq!(plan.effective_params["minimum_mean_coverage"], 0.75);
    Ok(())
}

#[test]
fn adna_stage_plans_preserve_evidence_only_contracts() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-adna-governance")?;
    let bam = temp.path().join("sample.bam");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, b"bam")?;
    std::fs::write(&reference, b">chr1\nACGT\n")?;

    let damage = plan_stage(StagePlanRequest {
        stage_id: BamStage::Damage.as_str(),
        tool: &dummy_tool("pydamage"),
        out_dir: temp.path(),
        bam: Some(&bam),
        bam_index: None,
        r1: None,
        r2: None,
        reference: Some(&reference),
        sample_id: Some("sample"),
        params: None,
    })?;
    assert_eq!(damage.effective_params["evidence_only"], true);
    assert_eq!(damage.effective_params["damage_tool_profile"], "ancient_dna_evidence");

    let authenticity = plan_stage(StagePlanRequest {
        stage_id: BamStage::Authenticity.as_str(),
        tool: &dummy_tool("pmdtools"),
        out_dir: temp.path(),
        bam: Some(&bam),
        bam_index: None,
        r1: None,
        r2: None,
        reference: Some(&reference),
        sample_id: Some("sample"),
        params: None,
    })?;
    assert_eq!(authenticity.effective_params["evidence_only"], true);
    assert_eq!(authenticity.effective_params["disallow_certification"], true);
    Ok(())
}

#[test]
fn planner_reason_details_include_resource_plan_and_endogenous_context() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-resource-governance")?;
    let bam = temp.path().join("sample.bam");
    let bam_index = temp.path().join("sample.bam.bai");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, vec![0_u8; 8 * 1024 * 1024])?;
    std::fs::write(&bam_index, b"bai")?;
    std::fs::write(&reference, b">chr1\nACGT\n")?;

    let markdup = plan_stage(StagePlanRequest {
        stage_id: BamStage::Markdup.as_str(),
        tool: &dummy_tool("samtools"),
        out_dir: temp.path(),
        bam: Some(&bam),
        bam_index: Some(&bam_index),
        r1: None,
        r2: None,
        reference: Some(&reference),
        sample_id: Some("sample"),
        params: None,
    })?;
    assert_eq!(markdup.reason.details["resource_plan"]["stage_id"], "bam.markdup");
    assert_eq!(markdup.reason.details["resource_plan"]["requires_index"], true);

    let endogenous = plan_stage(StagePlanRequest {
        stage_id: BamStage::EndogenousContent.as_str(),
        tool: &dummy_tool("samtools"),
        out_dir: temp.path(),
        bam: Some(&bam),
        bam_index: Some(&bam_index),
        r1: None,
        r2: None,
        reference: Some(&reference),
        sample_id: Some("sample"),
        params: Some(&serde_json::json!({
            "regions": null,
            "depth_thresholds": [1, 3, 5],
            "host_reference_scope": "human_host",
            "host_reference_digest": "sha256:host",
            "refuse_without_host_reference": true
        })),
    })?;
    assert_eq!(endogenous.effective_params["host_reference_scope"], "human_host");
    assert_eq!(endogenous.params["host_reference_scope"], "human_host");
    Ok(())
}

#[test]
fn ancient_like_template_stage_plans_include_scientific_advisory_chain() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-template-governance")?;
    let bam = temp.path().join("sample.bam");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, b"bam")?;
    std::fs::write(&reference, b">chrM\nACGT\n")?;

    let mut tool_specs = BTreeMap::new();
    for stage_id in bijux_dna_planner_bam::pipeline_id_catalog("bam-to-bam__adna_shotgun__v1") {
        let stage = BamStage::try_from(stage_id.as_str())?;
        let tool_id = bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage);
        tool_specs.insert(stage_id, dummy_tool(tool_id.as_str()));
    }

    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs,
        params_overrides: BTreeMap::new(),
        bam,
        bam_index: None,
        reference: Some(reference),
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    };
    let plans = plan_bam_workflow_template_stage_plans("bam.essential_ancient_like", &inputs)?;
    let stage_ids: Vec<&str> = plans.iter().map(|plan| plan.stage_id.0.as_ref()).collect();
    assert!(stage_ids.contains(&"bam.damage"));
    assert!(stage_ids.contains(&"bam.authenticity"));
    assert!(stage_ids.contains(&"bam.contamination"));
    Ok(())
}
