use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use bijux_core::plan::execution_plan::{ExecutionPlan, PlanPolicy};
use bijux_core::plan::stage_plan::{
    ArtifactRef, CommandSpecV1, ContainerImageRefV1, PlanDecisionReason, StageIO, StagePlanV1,
};
use bijux_core::primitives::hashing::params_hash;
use bijux_core::{PipelineId, StageId, StageVersion, ToolConstraints, ToolId};
use bijux_pipelines::DefaultsLedgerV1;
use bijux_runner::{Artifact, Invocation, Runner, RunnerResult};
use bijux_runtime::recording::write_plan_provenance;
use bijux_runtime::FactsRowV1;

struct FakeRunner;

impl Runner for FakeRunner {
    fn run(&self, invocation: &Invocation) -> anyhow::Result<RunnerResult> {
        Ok(RunnerResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::from_millis(1),
            artifacts: vec![Artifact {
                path: invocation.stage.out_dir.join("output.txt"),
                sha256: "deadbeef".to_string(),
            }],
        })
    }
}

fn build_plan(base_dir: &Path) -> Result<ExecutionPlan> {
    let stage = StagePlanV1 {
        stage_id: StageId::from_static("core.test"),
        stage_version: StageVersion(1),
        tool_id: ToolId::from_static("tool.test"),
        tool_version: "0.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "example/tool:test".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), "hello".to_string()],
        },
        resources: ToolConstraints {
            runtime: "1h".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "input".to_string(),
                path: base_dir.join("input.fq"),
            }],
            outputs: vec![ArtifactRef {
                name: "output".to_string(),
                path: base_dir.join("output.fq"),
            }],
        },
        out_dir: base_dir.join("out"),
        params: serde_json::json!({"k": 1}),
        effective_params: serde_json::json!({"k": 1}),
        aux_images: BTreeMap::new(),
        reason: PlanDecisionReason::default(),
    };

    ExecutionPlan::new(
        "pipeline.test",
        "planner.test",
        PlanPolicy::PreferAccuracy,
        vec![stage],
        Vec::new(),
    )
}

#[test]
fn golden_spine_contract() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let base_dir = tmp.path();
    let plan = build_plan(base_dir)?;

    bijux_engine::validate(&plan)?;

    let runner = FakeRunner;
    let environment = bijux_runtime::environment::ExecutionEnvironment;
    let _record = bijux_engine::execute(&plan, &runner, &environment, base_dir)?;

    let provenance_path = write_plan_provenance(base_dir, &plan)?;
    assert!(provenance_path.exists());

    let plan_hash = plan.plan_hash()?;
    let plan_hash_path = base_dir.join("plan_hash.txt");
    bijux_infra::write_bytes(&plan_hash_path, plan_hash.as_bytes())?;
    assert!(plan_hash_path.exists());

    let params_hash = params_hash(&serde_json::json!({"k": 1}))?;
    let facts_row = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-test".to_string(),
        stage_id: "core.test".to_string(),
        tool_id: "tool.test".to_string(),
        tool_version: "0.0.0".to_string(),
        image_digest: Some("sha256:deadbeef".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash,
        input_hash: "input".to_string(),
        output_hashes: vec!["output".to_string()],
        runtime_s: 1.0,
        memory_mb: 32.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(1),
        reads_out: Some(1),
        bases_in: Some(1),
        bases_out: Some(1),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({"reads_in": 1, "reads_out": 1}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    };

    let facts_path = base_dir.join("facts.jsonl");
    let facts_line = format!("{}\n", serde_json::to_string(&facts_row)?);
    bijux_infra::write_bytes(&facts_path, facts_line.as_bytes())?;

    let defaults_path = base_dir.join("defaults_ledger.json");
    let defaults = DefaultsLedgerV1 {
        pipeline_id: PipelineId::new("pipeline.test"),
        tools: BTreeMap::new(),
        params: BTreeMap::new(),
        thresholds: BTreeMap::new(),
        tool_provenance: BTreeMap::new(),
        param_provenance: BTreeMap::new(),
        assumptions: Vec::new(),
        citations: BTreeMap::new(),
    };
    bijux_infra::write_bytes(&defaults_path, serde_json::to_vec(&defaults)?)?;

    let report_path = bijux_analyze::write_run_report_from_facts(base_dir, &[facts_row])?;
    assert!(report_path.exists());

    let report_json: serde_json::Value = serde_json::from_slice(&std::fs::read(&report_path)?)?;
    let index_html = bijux_api::v1::report::render_report_bundle_html(&report_json);
    let bundle_dir = base_dir.join("report_bundle");
    bijux_infra::ensure_dir(&bundle_dir)?;
    let index_path = bundle_dir.join("index.html");
    bijux_infra::write_bytes(&index_path, index_html.as_bytes())?;
    assert!(index_path.exists());

    let provenance_raw = std::fs::read(&provenance_path)?;
    let provenance_json: serde_json::Value = serde_json::from_slice(&provenance_raw)?;
    assert!(provenance_json["tools"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|tool| { tool["image_digest"] == "sha256:deadbeef" }));

    Ok(())
}
