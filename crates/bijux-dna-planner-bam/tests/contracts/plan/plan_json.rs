use anyhow::Result;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::{plan_stage, StagePlanRequest};
use bijux_dna_stage_contract::StagePlanV1;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-planner-bam__{group}__{name}")
}

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

fn stage_snapshot_id(stage: BamStage) -> &'static str {
    match stage {
        BamStage::Align => "stage__bam__bam.align",
        BamStage::Validate => "stage__bam__bam.validate",
        BamStage::QcPre => "stage__bam__bam.qc_pre",
        BamStage::Filter => "stage__bam__bam.filter",
        BamStage::Markdup => "stage__bam__bam.markdup",
        BamStage::Complexity => "stage__bam__bam.complexity",
        BamStage::Coverage => "stage__bam__bam.coverage",
        BamStage::Damage => "stage__bam__bam.damage",
        BamStage::Authenticity => "stage__bam__bam.authenticity",
        BamStage::Contamination => "stage__bam__bam.contamination",
        BamStage::Sex => "stage__bam__bam.sex",
        BamStage::BiasMitigation => "stage__bam__bam.bias_mitigation",
        BamStage::Recalibration => "stage__bam__bam.recalibration",
        BamStage::Haplogroups => "stage__bam__bam.haplogroups",
        BamStage::Genotyping => "stage__bam__bam.genotyping",
        BamStage::Kinship => "stage__bam__bam.kinship",
        _ => "stage__bam__bam.align",
    }
}

fn tool_id_from_snapshot(stage: BamStage) -> Option<ToolId> {
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{}.json", snapshot_name("contracts", stage_snapshot_id(stage))));
    let raw = fs::read_to_string(snapshot_path).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    parsed.get("tool_id").and_then(Value::as_str).map(ToolId::new)
}

fn plan_for_stage(stage: BamStage) -> Result<StagePlanV1> {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("plan_inputs")
        .join("default");
    // Avoid any dependency on process CWD in concurrent test runs.
    let out_dir = fixtures.join("out");
    let tool_id = tool_id_from_snapshot(stage)
        .unwrap_or_else(|| bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage));
    let tool = dummy_tool(tool_id.as_str());
    plan_stage(StagePlanRequest {
        stage_id: stage.as_str(),
        tool: &tool,
        out_dir: out_dir.as_path(),
        bam: Some(&fixtures.join("sample.bam")),
        bam_index: Some(&fixtures.join("sample.bam.bai")),
        r1: Some(&fixtures.join("reads_R1.fastq.gz")),
        r2: Some(&fixtures.join("reads_R2.fastq.gz")),
        reference: Some(&fixtures.join("reference.fasta")),
        sample_id: Some("sample"),
        params: None,
    })
}

fn has_tool_candidates(stage: BamStage) -> bool {
    bijux_dna_domain_bam::stage_contract_json(stage.as_str())
        .and_then(|value| value.get("tool_ids").cloned())
        .and_then(|value| value.as_array().cloned())
        .is_some_and(|ids| !ids.is_empty())
}

fn sort_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, sort_json(v))).collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_json).collect()),
        other => other,
    }
}

fn assert_snapshot(name: &str, plan: &StagePlanV1) -> Result<()> {
    let canonical = sort_json(serde_json::to_value(plan)?);
    let mut payload = serde_json::to_string_pretty(&canonical)?;
    let crate_root = format!("{}/", PathBuf::from(env!("CARGO_MANIFEST_DIR")).display());
    payload = payload.replace(&crate_root, "");
    payload = payload.replace("tests/fixtures/plan_inputs/default/out/", "out/");
    payload = payload.replace("tests/fixtures/plan_inputs/default/out", "out");
    payload = payload.replace("\"tests/fixtures/plan_inputs/default/out\"", "\"out\"");
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{}.json", snapshot_name("contracts", name)));
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        fs::write(&snapshot_path, payload)?;
        return Ok(());
    }
    let expected_raw = fs::read_to_string(&snapshot_path)?;
    let expected_json: Value = serde_json::from_str(&expected_raw)?;
    let expected = serde_json::to_string_pretty(&sort_json(expected_json))?;
    assert_eq!(payload.trim_end_matches('\n'), expected.trim_end_matches('\n'));
    Ok(())
}

#[test]
fn stage_plan_snapshots_are_stable() -> Result<()> {
    let plan = plan_for_stage(BamStage::Align)?;
    assert_snapshot("stage__bam__bam.align", &plan)?;

    let plan = plan_for_stage(BamStage::Validate)?;
    assert_snapshot("stage__bam__bam.validate", &plan)?;

    let plan = plan_for_stage(BamStage::QcPre)?;
    assert_snapshot("stage__bam__bam.qc_pre", &plan)?;

    let plan = plan_for_stage(BamStage::Filter)?;
    assert_snapshot("stage__bam__bam.filter", &plan)?;

    let plan = plan_for_stage(BamStage::Markdup)?;
    assert_snapshot("stage__bam__bam.markdup", &plan)?;

    let plan = plan_for_stage(BamStage::Complexity)?;
    assert_snapshot("stage__bam__bam.complexity", &plan)?;

    let plan = plan_for_stage(BamStage::Coverage)?;
    assert_snapshot("stage__bam__bam.coverage", &plan)?;

    let plan = plan_for_stage(BamStage::Damage)?;
    assert_snapshot("stage__bam__bam.damage", &plan)?;

    let plan = plan_for_stage(BamStage::Authenticity)?;
    assert_snapshot("stage__bam__bam.authenticity", &plan)?;

    let plan = plan_for_stage(BamStage::Contamination)?;
    assert_snapshot("stage__bam__bam.contamination", &plan)?;

    let plan = plan_for_stage(BamStage::Sex)?;
    assert_snapshot("stage__bam__bam.sex", &plan)?;

    if cfg!(feature = "bam_downstream") && has_tool_candidates(BamStage::BiasMitigation) {
        let plan = plan_for_stage(BamStage::BiasMitigation)?;
        assert_snapshot("stage__bam__bam.bias_mitigation", &plan)?;
    }

    let plan = plan_for_stage(BamStage::Recalibration)?;
    assert_snapshot("stage__bam__bam.recalibration", &plan)?;

    if cfg!(feature = "bam_downstream") && has_tool_candidates(BamStage::Haplogroups) {
        let plan = plan_for_stage(BamStage::Haplogroups)?;
        assert_snapshot("stage__bam__bam.haplogroups", &plan)?;
    }

    if cfg!(feature = "bam_downstream") && has_tool_candidates(BamStage::Genotyping) {
        let plan = plan_for_stage(BamStage::Genotyping)?;
        assert_snapshot("stage__bam__bam.genotyping", &plan)?;
    }

    if cfg!(feature = "bam_downstream") && has_tool_candidates(BamStage::Kinship) {
        let plan = plan_for_stage(BamStage::Kinship)?;
        assert_snapshot("stage__bam__bam.kinship", &plan)?;
    }
    Ok(())
}

#[test]
fn plan_stage_rejects_tool_from_wrong_stage() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("plan_inputs")
        .join("default");
    let tool = dummy_tool("fastp");
    let result = plan_stage(StagePlanRequest {
        stage_id: BamStage::Align.as_str(),
        tool: &tool,
        out_dir: fixtures.join("out").as_path(),
        bam: Some(&fixtures.join("sample.bam")),
        bam_index: Some(&fixtures.join("sample.bam.bai")),
        r1: Some(&fixtures.join("reads_R1.fastq.gz")),
        r2: Some(&fixtures.join("reads_R2.fastq.gz")),
        reference: Some(&fixtures.join("reference.fasta")),
        sample_id: Some("sample"),
        params: None,
    });
    assert!(result.is_err(), "planner accepted a tool that is not registered for bam.align");
}
