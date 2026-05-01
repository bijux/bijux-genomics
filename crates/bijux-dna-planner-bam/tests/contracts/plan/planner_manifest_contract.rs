use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_core::contract::{
    build_plan_manifest, planner_refusal_from_message, ArtifactRole, ParameterResolutionTraceV1,
    PlanManifestBuildInputV1, PlanPolicy, PlannerParameterSourceV1, PlannerWarningCodeV1,
    PlannerWarningRecordV1, WorkflowInputArtifactV1, WorkflowManifestV1, WorkflowReferenceAssetV1,
    WorkflowStageRequestV1,
};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::{plan_bam_stage_plans_for_profile_id, BamPipelineInputs};
use insta::Settings;

/// Snapshot intent: keep BAM planner manifests deterministic and reviewable.
fn snapshot_name(name: &str) -> String {
    format!("bijux-dna-planner-bam__contracts__{name}")
}

fn snapshot_settings(temp_path: &Path) -> Settings {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.add_filter(temp_path.to_str().unwrap_or_default(), "<temp>");
    settings
}

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "0.7.17".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/bwa".to_string(),
            digest: Some("sha256:bwa".to_string()),
        },
        command: CommandSpecV1 { template: vec!["bwa".to_string()] },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 2,
            tmp_gb: 1,
            threads: 2,
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

fn workflow_manifest(advisory_coverage: bool) -> WorkflowManifestV1 {
    let mut manifest = WorkflowManifestV1::new("bam", "bam-to-bam__default__v1");
    manifest.inputs = vec![WorkflowInputArtifactV1 {
        artifact_id: "bam".to_string(),
        role: ArtifactRole::Bam,
        path: PathBuf::from("sample.bam"),
        layout: None,
        compression: None,
        format_id: Some("bam".to_string()),
    }];
    manifest.reference_assets = vec![WorkflowReferenceAssetV1 {
        asset_id: "reference".to_string(),
        role: ArtifactRole::Reference,
        path: PathBuf::from("reference.fasta"),
        checksum_sha256: Some("c".repeat(64)),
        build_id: Some("GRCh38".to_string()),
        alias_group: None,
    }];
    manifest.requested_stages = vec![
        WorkflowStageRequestV1 { stage_id: "bam.validate".to_string(), advisory_only: false },
        WorkflowStageRequestV1 {
            stage_id: "bam.coverage".to_string(),
            advisory_only: advisory_coverage,
        },
    ];
    manifest
}

fn plan_manifest_payload(temp: &Path, advisory_coverage: bool) -> Result<serde_json::Value> {
    let bam = temp.join("sample.bam");
    let reference = temp.join("reference.fasta");
    std::fs::write(&bam, b"")?;
    std::fs::write(&reference, b">chrM\nACGT\n")?;
    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: tool_specs_for_profile("bam-to-bam__default__v1"),
        params_overrides: BTreeMap::new(),
        bam,
        bam_index: None,
        reference: Some(reference),
        sample_id: Some("sample".to_string()),
        out_dir: temp.join("out"),
        allow_planned: false,
    };
    let graph = bijux_dna_planner_bam::plan_bam_to_bam__default__v1(&inputs)?;
    let stage_plans = plan_bam_stage_plans_for_profile_id("bam-to-bam__default__v1", &inputs)?;
    let effective_parameters_by_step = stage_plans
        .iter()
        .map(|plan| {
            (
                plan.stage_instance_id
                    .as_ref()
                    .map_or_else(|| plan.stage_id.to_string(), std::string::ToString::to_string),
                plan.effective_params.clone(),
            )
        })
        .collect();
    let parameter_traces = stage_plans
        .iter()
        .filter_map(|plan| {
            let serde_json::Value::Object(map) = &plan.effective_params else {
                return None;
            };
            let (name, value) = map.iter().next()?;
            Some(ParameterResolutionTraceV1 {
                step_id: plan
                    .stage_instance_id
                    .as_ref()
                    .map_or_else(|| plan.stage_id.to_string(), std::string::ToString::to_string),
                stage_id: plan.stage_id.to_string(),
                parameter: name.clone(),
                source: PlannerParameterSourceV1::PlannerInferred,
                resolved_value: value.clone(),
                detail: "derived from stage effective params".to_string(),
            })
        })
        .collect();
    let warnings = if advisory_coverage {
        vec![PlannerWarningRecordV1 {
            code: PlannerWarningCodeV1::AdvisoryStage,
            stage_id: Some("bam.coverage".to_string()),
            message: "coverage report treated as advisory in this contract".to_string(),
        }]
    } else {
        Vec::new()
    };
    let manifest = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow_manifest(advisory_coverage),
        graph,
        stage_contract_refs: Vec::new(),
        effective_parameters_by_step,
        parameter_traces,
        refusal_records: Vec::new(),
        warning_records: warnings,
    })?;
    Ok(serde_json::to_value(manifest)?)
}

#[test]
fn bam_happy_plan_manifest_snapshot_is_stable() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-plan-manifest-happy")?;
    let settings = snapshot_settings(temp.path());
    settings.bind(|| {
        insta::assert_json_snapshot!(
            snapshot_name("bam_happy_plan_manifest"),
            bijux_dna_testkit::snapshot_normalize_json(
                &plan_manifest_payload(temp.path(), false).expect("plan manifest")
            )
        );
    });
    Ok(())
}

#[test]
fn bam_advisory_plan_manifest_snapshot_is_stable() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-plan-manifest-advisory")?;
    let settings = snapshot_settings(temp.path());
    settings.bind(|| {
        insta::assert_json_snapshot!(
            snapshot_name("bam_advisory_plan_manifest"),
            bijux_dna_testkit::snapshot_normalize_json(
                &plan_manifest_payload(temp.path(), true).expect("plan manifest")
            )
        );
    });
    Ok(())
}

#[test]
fn bam_refusal_manifest_snapshot_is_stable() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-plan-manifest-refusal")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;
    let mut inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: BTreeMap::new(),
        params_overrides: BTreeMap::new(),
        bam,
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    };
    inputs.tool_specs.insert("vcf.call".to_string(), dummy_tool("bcftools"));
    let error = bijux_dna_planner_bam::plan_bam_to_bam__default__v1(&inputs)
        .expect_err("foreign tool spec should fail");
    let refusal = planner_refusal_from_message(None, &error.to_string());
    let payload = serde_json::json!({
        "workflow_manifest": workflow_manifest(false),
        "refusal_records": [refusal],
    });
    let settings = snapshot_settings(temp.path());
    settings.bind(|| {
        insta::assert_json_snapshot!(
            snapshot_name("bam_refusal_manifest"),
            bijux_dna_testkit::snapshot_normalize_json(&payload)
        );
    });
    Ok(())
}
