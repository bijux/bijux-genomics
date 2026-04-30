use std::collections::BTreeMap;

use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_runtime::observability::RunProvenanceV1;
use bijux_dna_runtime::run_layout::{
    ExecutorDescriptorV1, RunExecutionModeV1, RunFailureV1, RunLayoutV1, RunLifecycleStateV1,
    RunManifest, RunStateV1,
};

#[path = "../../support/workspace_paths.rs"]
mod support;

#[test]
fn run_layout_schema_snapshot() {
    let layout = RunLayoutV1 {
        schema_version: "bijux.run_layout.v1".to_string(),
        run_dir: "run".to_string(),
        stages_dir: "stages".to_string(),
        manifests_dir: "manifests".to_string(),
        logs_dir: "logs".to_string(),
        reports_dir: "reports".to_string(),
        summary_dir: "summary".to_string(),
        run_artifacts_dir: "run_artifacts".to_string(),
        checkpoints_dir: "checkpoints".to_string(),
        assessment_path: "input_assessment.json".to_string(),
        graph_path: "graph.json".to_string(),
        plan_manifest_path: "plan_manifest.json".to_string(),
        manifest_path: "run_manifest.json".to_string(),
        environment_path: "environment.json".to_string(),
        metadata_path: "run_metadata.json".to_string(),
        events_path: "events.jsonl".to_string(),
        run_state_path: "run_state.json".to_string(),
        runtime_policy_path: "runtime_policy.json".to_string(),
        executor_descriptor_path: "executor_descriptor.json".to_string(),
        checkpoint_path: "checkpoint.json".to_string(),
        failure_path: "run_failure.json".to_string(),
        run_summary_path: "run_summary.json".to_string(),
        run_summary_text_path: "run_summary.txt".to_string(),
        artifact_inventory_path: "artifact_inventory.json".to_string(),
        artifact_inventory_text_path: "artifact_inventory.txt".to_string(),
        replay_manifest_path: "replay_manifest.json".to_string(),
        hash_ledger_path: "hash_ledger.json".to_string(),
        evidence_verification_path: "evidence_verification.json".to_string(),
        evidence_bundle_path: "evidence_bundle.json".to_string(),
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_layout_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&layout)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn run_record_schema_snapshot() {
    let record = bijux_dna_core::contract::RunRecordV1::new(vec![
        bijux_dna_core::contract::StageExecutionRecordV1 {
            stage_id: "fastq.trim_reads".to_string(),
            attempt: 0,
            success: true,
            cached: false,
        },
    ]);
    let expected = include_str!("../../fixtures/runtime_schema/default/run_record_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&record)
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
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&provenance)
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
        stage_id: bijux_dna_core::ids::StageId::new("fastq.trim_reads"),
        tool_id: bijux_dna_core::ids::ToolId::new("fastp"),
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
        resources: bijux_dna_core::contract::ToolConstraints::default(),
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
        layout: bijux_dna_core::prelude::input_assessment::FastqLayout::SingleEnd,
        stages: Vec::new(),
        tool_invocations: vec![invocation],
        artifacts: Vec::new(),
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_manifest_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual, expected);
}

#[test]
fn run_state_schema_snapshot() {
    let state = RunStateV1 {
        schema_version: "bijux.run_state.v1".to_string(),
        run_id: "run-1".to_string(),
        mode: RunExecutionModeV1::Enforced,
        state: RunLifecycleStateV1::Succeeded,
        transitions: Vec::new(),
        manifest_path: Some("run_manifest.json".into()),
        checkpoint_path: Some("checkpoints/checkpoint.json".into()),
        failure_path: None,
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_state_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&state)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn run_failure_schema_snapshot() {
    let failure = RunFailureV1 {
        schema_version: "bijux.run_failure.v1".to_string(),
        run_id: "run-1".to_string(),
        mode: RunExecutionModeV1::Enforced,
        state: RunLifecycleStateV1::Failed,
        failure_code: "runner_execution_failed".to_string(),
        message: "step failed after retries: fastq.validate_reads".to_string(),
        stage_id: Some("fastq.validate_reads".to_string()),
        step_id: Some("fastq.validate_reads".to_string()),
        attempt: None,
        observed_at: "2024-01-01T00:00:10Z".to_string(),
        retryable: true,
    };
    let expected = include_str!("../../fixtures/runtime_schema/default/run_failure_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&failure)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn executor_descriptor_schema_snapshot() {
    let descriptor = ExecutorDescriptorV1::Hpc {
        scheduler: "slurm".to_string(),
        submission_mode: "batch".to_string(),
        scratch_layout_policy: "stage_scoped_scratch".to_string(),
        container_runtime: Some("apptainer".to_string()),
    };
    let expected =
        include_str!("../../fixtures/runtime_schema/default/executor_descriptor_v1.json");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&descriptor)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn schema_fixture_names_include_version() {
    let dir = support::crate_root("bijux-dna-runtime")
        .unwrap_or_else(|err| panic!("resolve runtime crate root: {err}"))
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
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| panic!("fixture name is not valid UTF-8: {}", path.display()));
        if !std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        if name == "CASE.json" {
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
