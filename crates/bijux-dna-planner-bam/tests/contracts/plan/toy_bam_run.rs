use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::{
    bam_workflow_template_catalog, plan_bam_to_bam__adna_shotgun__v1, plan_bam_to_bam__default__v1,
    plan_bam_workflow_template, BamPipelineInputs,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "fixture.1".to_string(),
        image: ContainerImageRefV1 {
            image: format!("bijux/{tool}:fixture"),
            digest: Some(format!("sha256:{tool}")),
        },
        command: CommandSpecV1 { template: vec![tool.to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn tool_specs_for_profile(profile_id: &str) -> BTreeMap<String, ToolExecutionSpecV1> {
    let mut specs = BTreeMap::new();
    for stage_id in bijux_dna_planner_bam::pipeline_id_catalog(profile_id) {
        let stage = BamStage::try_from(stage_id.as_str()).expect("stage id");
        let tool_id = bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage);
        specs.insert(stage_id, dummy_tool(tool_id.as_str()));
    }
    specs
}

#[test]
fn toy_bam_run_plans_adna_pipeline_with_damage_stage() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("toy-bam-run")?;
    let bam = temp.path().join("toy.bam");
    let bai = temp.path().join("toy.bam.bai");
    let reference = temp.path().join("toy.fasta");

    // Minimal local synthetic fixtures for planning-only integration.
    std::fs::write(&bam, b"BAM\x01")?;
    std::fs::write(&bai, b"BAI\x01")?;
    std::fs::write(&reference, b">chrM\nACGTACGT\n")?;

    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: tool_specs_for_profile("bam-to-bam__adna_shotgun__v1"),
        params_overrides: BTreeMap::new(),
        bam,
        bam_index: Some(bai),
        reference: Some(reference),
        sample_id: Some("toy".to_string()),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    };

    let graph = plan_bam_to_bam__adna_shotgun__v1(&inputs)?;
    let stage_ids =
        graph.steps().iter().map(|step| step.stage_id.as_str().to_string()).collect::<Vec<_>>();

    assert!(stage_ids.iter().any(|id| id == "bam.validate"));
    assert!(stage_ids.iter().any(|id| id == "bam.coverage"));
    assert!(stage_ids.iter().any(|id| id == "bam.damage"));
    assert!(stage_ids.len() >= 5, "expected non-trivial BAM plan with core stages");

    Ok(())
}

#[test]
fn toy_bam_run_plans_default_pipeline_without_ancient_advisory_stages() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("toy-bam-default-run")?;
    let bam = temp.path().join("toy.bam");
    let bai = temp.path().join("toy.bam.bai");
    let reference = temp.path().join("toy.fasta");

    std::fs::write(&bam, b"BAM\x01")?;
    std::fs::write(&bai, b"BAI\x01")?;
    std::fs::write(&reference, b">chrM\nACGTACGT\n")?;

    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: tool_specs_for_profile("bam-to-bam__default__v1"),
        params_overrides: BTreeMap::new(),
        bam,
        bam_index: Some(bai),
        reference: Some(reference),
        sample_id: Some("toy".to_string()),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    };

    let graph = plan_bam_to_bam__default__v1(&inputs)?;
    let stage_ids =
        graph.steps().iter().map(|step| step.stage_id.as_str().to_string()).collect::<Vec<_>>();

    assert!(stage_ids.iter().any(|id| id == "bam.validate"));
    assert!(stage_ids.iter().any(|id| id == "bam.mapping_summary"));
    assert!(stage_ids.iter().any(|id| id == "bam.coverage"));
    assert!(stage_ids.iter().any(|id| id == "bam.damage"));
    assert!(!stage_ids.iter().any(|id| id == "bam.authenticity"));
    assert!(!stage_ids.iter().any(|id| id == "bam.contamination"));

    Ok(())
}

#[test]
fn bam_workflow_templates_select_modern_and_ancient_stage_sets_explicitly() -> Result<()> {
    let templates = bam_workflow_template_catalog();
    assert!(templates.iter().any(|template| template.template_id == "bam.essential_modern"));
    assert!(templates.iter().any(|template| template.template_id == "bam.essential_ancient_like"));

    let temp = bijux_dna_infra::temp_dir("toy-bam-template-run")?;
    let bam = temp.path().join("toy.bam");
    let bai = temp.path().join("toy.bam.bai");
    let reference = temp.path().join("toy.fasta");
    std::fs::write(&bam, b"BAM\x01")?;
    std::fs::write(&bai, b"BAI\x01")?;
    std::fs::write(&reference, b">chrM\nACGTACGT\n")?;

    let modern_inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: tool_specs_for_profile("bam-to-bam__default__v1"),
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: Some(bai.clone()),
        reference: Some(reference.clone()),
        sample_id: Some("toy".to_string()),
        out_dir: temp.path().join("modern-out"),
        allow_planned: false,
    };
    let modern = plan_bam_workflow_template("bam.essential_modern", &modern_inputs)?;
    let modern_stage_ids =
        modern.steps().iter().map(|step| step.stage_id.as_str().to_string()).collect::<Vec<_>>();
    assert!(modern_stage_ids.iter().any(|id| id == "bam.validate"));
    assert!(modern_stage_ids.iter().any(|id| id == "bam.mapping_summary"));
    assert!(modern_stage_ids.iter().any(|id| id == "bam.coverage"));
    assert!(modern_stage_ids.iter().any(|id| id == "bam.damage"));
    assert!(!modern_stage_ids.iter().any(|id| id == "bam.align"));
    assert!(!modern_stage_ids.iter().any(|id| id == "bam.authenticity"));
    assert!(!modern_stage_ids.iter().any(|id| id == "bam.contamination"));

    let ancient_inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: tool_specs_for_profile("bam-to-bam__adna_shotgun__v1"),
        params_overrides: BTreeMap::new(),
        bam,
        bam_index: Some(bai),
        reference: Some(reference),
        sample_id: Some("toy".to_string()),
        out_dir: temp.path().join("ancient-out"),
        allow_planned: false,
    };
    let ancient = plan_bam_workflow_template("bam.essential_ancient_like", &ancient_inputs)?;
    let ancient_stage_ids =
        ancient.steps().iter().map(|step| step.stage_id.as_str().to_string()).collect::<Vec<_>>();
    assert!(ancient_stage_ids.iter().any(|id| id == "bam.validate"));
    assert!(ancient_stage_ids.iter().any(|id| id == "bam.damage"));
    assert!(ancient_stage_ids.iter().any(|id| id == "bam.authenticity"));
    assert!(ancient_stage_ids.iter().any(|id| id == "bam.contamination"));
    Ok(())
}
