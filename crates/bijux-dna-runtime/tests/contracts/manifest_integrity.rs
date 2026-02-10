use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_runtime::run_layout::RunManifest;
use bijux_dna_runtime::{prepare_tool_run_dirs, write_canonical_json, write_run_manifest};

#[test]
fn manifest_has_required_fields() {
    let manifest = RunManifest {
        schema_version: "bijux.run_manifest.v3".to_string(),
        contract_version: bijux_dna_core::contract::ContractVersion { major: 1, minor: 0 },
        run_id: "run-1".to_string(),
        started_at: "2024-01-01T00:00:00Z".to_string(),
        finished_at: "2024-01-01T00:00:10Z".to_string(),
        pipeline: "fastq-to-fastq__default__v1".to_string(),
        graph_hash: "sha256:graph".to_string(),
        cache_key: None,
        layout: bijux_dna_core::prelude::input_assessment::FastqLayout::SingleEnd,
        stages: Vec::new(),
        tool_invocations: vec![bijux_dna_core::metrics::ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            contract_version: bijux_dna_core::contract::ContractVersion { major: 1, minor: 0 },
            stage_id: bijux_dna_core::ids::StageId::new("fastq.trim"),
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
        }],
        artifacts: Vec::new(),
    };
    let json = serde_json::to_value(&manifest).unwrap_or_else(|err| panic!("serialize: {err}"));
    for key in [
        "schema_version",
        "contract_version",
        "run_id",
        "started_at",
        "finished_at",
        "pipeline",
        "graph_hash",
        "layout",
        "stages",
        "tool_invocations",
        "artifacts",
    ] {
        assert!(json.get(key).is_some(), "missing {key}");
    }
    let invocations = json
        .get("tool_invocations")
        .and_then(|value| value.as_array())
        .unwrap_or_else(|| panic!("tool_invocations array missing"));
    assert!(!invocations.is_empty(), "tool_invocations empty");
    let first = &invocations[0];
    assert!(
        first
            .get("input_hashes")
            .and_then(|value| value.as_array())
            .is_some(),
        "tool_invocation missing input_hashes"
    );
    assert!(
        first
            .get("tool_version")
            .and_then(|value| value.as_str())
            .is_some_and(|v| !v.trim().is_empty()),
        "tool_invocation missing tool_version"
    );
    assert!(
        first
            .get("image_digest")
            .and_then(|value| value.as_str())
            .is_some_and(|v| !v.trim().is_empty()),
        "tool_invocation missing image_digest"
    );
}

#[test]
fn run_manifest_output_artifacts_include_hashes_for_runtime_files() {
    let base = std::env::var("TEST_TMP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("runtime_manifest_hash_contract");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap_or_else(|e| panic!("create base dir: {e}"));
    let run_dirs = prepare_tool_run_dirs(&base, "fastp", "run-1")
        .unwrap_or_else(|e| panic!("prepare run dirs: {e}"));
    write_canonical_json(&run_dirs.manifest_path, &serde_json::json!({"ok": true}))
        .unwrap_or_else(|e| panic!("write manifest: {e}"));
    write_canonical_json(&run_dirs.metrics_path, &serde_json::json!({"metrics": []}))
        .unwrap_or_else(|e| panic!("write metrics: {e}"));
    let rp = bijux_dna_runtime::RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        pipeline_id: "fastq".to_string(),
        tool_version: "1.0.0".to_string(),
        tool_image_digest: Some("sha256:synthetic".to_string()),
        params_hash: "sha256:params".to_string(),
        input_hashes: vec!["sha256:in".to_string()],
        reference_genome: None,
        git_commit: "abc1234".to_string(),
        build_profile: "test".to_string(),
        plan_hash: Some("sha256:plan".to_string()),
    };
    write_run_manifest(&run_dirs, "fastq.trim", "fastp", &rp, None, &[])
        .unwrap_or_else(|e| panic!("write run manifest: {e}"));
    let raw = std::fs::read_to_string(&run_dirs.run_manifest_path)
        .unwrap_or_else(|e| panic!("read run manifest: {e}"));
    let value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse run manifest: {e}"));
    let artifacts = value
        .get("output_artifacts")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("missing output_artifacts"));
    assert!(!artifacts.is_empty(), "output_artifacts must not be empty");
    for item in artifacts {
        let hash = item
            .get("sha256")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        assert_eq!(
            hash.len(),
            64,
            "artifact hash must be 64-char sha256 hex, got {hash:?}"
        );
    }
}

#[test]
fn run_manifest_writes_reproducibility_report_artifact() {
    let base = std::env::var("TEST_TMP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("runtime_repro_report_contract");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap_or_else(|e| panic!("create base dir: {e}"));
    let run_dirs = prepare_tool_run_dirs(&base, "fastp", "run-1")
        .unwrap_or_else(|e| panic!("prepare run dirs: {e}"));
    write_canonical_json(&run_dirs.manifest_path, &serde_json::json!({"ok": true}))
        .unwrap_or_else(|e| panic!("write manifest: {e}"));
    write_canonical_json(&run_dirs.metrics_path, &serde_json::json!({"metrics": []}))
        .unwrap_or_else(|e| panic!("write metrics: {e}"));
    let rp = bijux_dna_runtime::RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        pipeline_id: "fastq".to_string(),
        tool_version: "1.0.0".to_string(),
        tool_image_digest: Some("sha256:synthetic".to_string()),
        params_hash: "sha256:params".to_string(),
        input_hashes: vec!["sha256:in".to_string()],
        reference_genome: None,
        git_commit: "abc1234".to_string(),
        build_profile: "test".to_string(),
        plan_hash: Some("sha256:plan".to_string()),
    };
    write_run_manifest(&run_dirs, "fastq.trim", "fastp", &rp, None, &[])
        .unwrap_or_else(|e| panic!("write run manifest: {e}"));
    let repro_path = run_dirs
        .manifest_path
        .parent()
        .unwrap_or_else(|| panic!("run dir missing"))
        .join("run_artifacts")
        .join("reproducibility")
        .join("report.json");
    assert!(repro_path.exists(), "reproducibility report missing");
    let raw = std::fs::read_to_string(&repro_path).unwrap_or_else(|e| panic!("read repro: {e}"));
    let value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse repro: {e}"));
    assert_eq!(
        value.get("schema_version").and_then(|v| v.as_str()),
        Some("bijux.reproducibility_report.v1")
    );
    assert!(value.get("plan_hash").is_some(), "missing plan_hash");
    assert!(value.get("input_hashes").is_some(), "missing input_hashes");
}
