use std::collections::BTreeMap;

use bijux_core::contract::ContractVersion;
use bijux_core::metrics::ToolInvocationV1;
use bijux_runtime::observability::RunProvenanceV1;
use bijux_runtime::run_layout::{RunLayoutV1, RunManifest};

#[test]
fn run_layout_schema_snapshot() {
    let layout = RunLayoutV1 {
        schema_version: "bijux.run_layout.v1".to_string(),
        run_dir: "/tmp/run".to_string(),
        stages_dir: "/tmp/run/stages".to_string(),
        summary_dir: "/tmp/run/summary".to_string(),
        assessment_path: "/tmp/run/input_assessment.json".to_string(),
        manifest_path: "/tmp/run/execution_manifest.json".to_string(),
        environment_path: "/tmp/run/environment.json".to_string(),
        metadata_path: "/tmp/run/run_metadata.json".to_string(),
        events_path: "/tmp/run/events.jsonl".to_string(),
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_layout_v1.json");
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&layout)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn run_record_schema_snapshot() {
    let record = bijux_core::contract::RunRecordV1::new(vec![
        bijux_core::contract::StageExecutionRecordV1 {
            stage_id: "fastq.trim".to_string(),
            attempt: 0,
            success: true,
            cached: false,
        },
    ]);
    let expected = include_str!("../../fixtures/runtime_schema/default/run_record_v1.json");
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&record)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn run_provenance_schema_snapshot() {
    let provenance = RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        tool_image_digest: Some("sha256:img".to_string()),
        tool_version: "1.0".to_string(),
        params_hash: "sha256:params".to_string(),
        input_hashes: vec!["sha256:input".to_string()],
        reference_genome: None,
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        git_commit: "abc".to_string(),
        build_profile: "dev".to_string(),
        plan_hash: None,
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_provenance_v1.json");
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&provenance)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn run_manifest_schema_snapshot() {
    let invocation = ToolInvocationV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: bijux_core::ids::StageId::new("fastq.trim"),
        tool_id: bijux_core::ids::ToolId::new("fastp"),
        tool_version: "1.0".to_string(),
        resolved_tool_version: None,
        image_digest: "sha256:img".to_string(),
        runner_kind: "docker".to_string(),
        platform: "local".to_string(),
        parameters_json: serde_json::json!({}),
        parameters_json_normalized: serde_json::json!({}),
        effective_params_json: serde_json::json!({}),
        effective_params_json_normalized: serde_json::json!({}),
        params_provenance: serde_json::json!({}),
        params_provenance_normalized: serde_json::json!({}),
        adapter_bank: None,
        banks: None,
        bank_assets: None,
        resources: bijux_core::contract::ToolConstraints::default(),
        environment: BTreeMap::default(),
        input_hashes: vec!["sha256:input".to_string()],
        output_hashes: vec!["sha256:output".to_string()],
        executed_command: None,
    };
    let manifest = RunManifest {
        schema_version: "bijux.run_manifest.v3".to_string(),
        contract_version: ContractVersion::v1(),
        run_id: "run-1".to_string(),
        started_at: "2024-01-01T00:00:00Z".to_string(),
        finished_at: "2024-01-01T00:00:10Z".to_string(),
        pipeline: "fastq-to-fastq__default__v1".to_string(),
        graph_hash: "sha256:graph".to_string(),
        cache_key: None,
        layout: bijux_core::prelude::input_assessment::FastqLayout::SingleEnd,
        stages: Vec::new(),
        tool_invocations: vec![invocation],
        artifacts: Vec::new(),
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_manifest_v1.json");
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&manifest)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn schema_fixture_names_include_version() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("runtime_schema")
        .join("default");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("read runtime_schema fixtures at {}: {err}", dir.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("fixture entry: {err}"));
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        if !name.ends_with("_v1.json") {
            offenders.push(name.to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "runtime schema fixtures must use *_v1.json names: {offenders:?}"
    );
}
