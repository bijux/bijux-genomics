use std::collections::BTreeMap;

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
        first.get("input_hashes").and_then(|value| value.as_array()).is_some(),
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
    let fixture = bijux_dna_testkit::tempdir_for("runtime_manifest_hash_contract");
    let base = fixture.path().to_path_buf();
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
    write_run_manifest(&run_dirs, "fastq.trim_reads", "fastp", &rp, None, &[])
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
            .unwrap_or_else(|| panic!("artifact missing sha256 string: {item}"));
        assert_eq!(hash.len(), 64, "artifact hash must be 64-char sha256 hex, got {hash:?}");
    }
}

#[test]
fn prepare_tool_run_dirs_rejects_path_traversal_segments() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_run_dirs_segment_contract");
    let base = fixture.path().to_path_buf();
    std::fs::create_dir_all(&base).unwrap_or_else(|e| panic!("create base dir: {e}"));

    for (tool, run_id) in [("../fastp", "run-1"), ("fastp", "../run-1"), ("", "run-1")] {
        let err = prepare_tool_run_dirs(&base, tool, run_id)
            .err()
            .unwrap_or_else(|| panic!("expected invalid segment for tool={tool:?} run={run_id:?}"));
        assert!(err.to_string().contains("path segment"), "unexpected segment error: {err}");
    }
}

#[test]
fn stage_plan_writer_rejects_paths_outside_plans_dir() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_stage_plan_path_contract");
    let base = fixture.path().to_path_buf();
    std::fs::create_dir_all(&base).unwrap_or_else(|e| panic!("create base dir: {e}"));
    let run_dirs = prepare_tool_run_dirs(&base, "fastp", "run-1")
        .unwrap_or_else(|e| panic!("prepare run dirs: {e}"));

    for file_name in ["../escape.json", "/tmp/escape.json"] {
        let err = bijux_dna_runtime::recording::write_stage_plan_json(
            &run_dirs,
            file_name,
            &serde_json::json!({"ok": true}),
        )
        .err()
        .unwrap_or_else(|| panic!("expected invalid stage plan path for {file_name:?}"));
        assert!(
            err.to_string().contains("relative path inside run_artifacts"),
            "unexpected stage plan path error: {err}"
        );
    }
}

#[test]
fn run_manifest_writes_reproducibility_report_artifact() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_repro_report_contract");
    let base = fixture.path().to_path_buf();
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
    write_run_manifest(&run_dirs, "fastq.trim_reads", "fastp", &rp, None, &[])
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

#[test]
fn run_manifest_writes_profile_and_lock_manifests() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_profile_lock_contract");
    let base = fixture.path().to_path_buf();
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
    write_run_manifest(&run_dirs, "fastq.trim_reads", "fastp", &rp, None, &[])
        .unwrap_or_else(|e| panic!("write run manifest: {e}"));
    let run_dir = run_dirs.run_manifest_path.parent().unwrap_or_else(|| panic!("run dir missing"));
    let profile_manifest_path = run_dir.join("profile_manifest.json");
    let lock_manifest_path = run_dir.join("run_manifest.lock.json");
    assert!(profile_manifest_path.exists(), "profile_manifest.json must exist");
    assert!(lock_manifest_path.exists(), "run_manifest.lock.json must exist");
}

#[test]
fn artifact_checksums_emit_deterministic_sha256_ledger() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_checksums_ledger_contract");
    let dir = fixture.path().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir: {e}"));
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    std::fs::write(&a, b"A").unwrap_or_else(|e| panic!("write a: {e}"));
    std::fs::write(&b, b"B").unwrap_or_else(|e| panic!("write b: {e}"));
    let _ = bijux_dna_runtime::recording::write_artifact_checksums_json(
        &dir,
        &[("a".to_string(), a), ("b".to_string(), b)],
    )
    .unwrap_or_else(|e| panic!("write checksums: {e}"));
    let sha_path = dir.join("checksums.sha256");
    assert!(sha_path.exists(), "checksums.sha256 must exist");
    let lines = std::fs::read_to_string(sha_path).unwrap_or_else(|e| panic!("read sha: {e}"));
    assert!(lines.contains("  a"));
    assert!(lines.contains("  b"));
}

#[test]
fn artifact_checksums_reject_declared_missing_artifacts() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_checksums_missing_contract");
    let dir = fixture.path().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir: {e}"));
    let missing = dir.join("missing.txt");

    let err = bijux_dna_runtime::recording::write_artifact_checksums_json(
        &dir,
        &[("missing".to_string(), missing)],
    )
    .err()
    .unwrap_or_else(|| panic!("expected missing artifact to fail"));

    assert!(
        err.to_string().contains("declared artifact `missing` is missing"),
        "unexpected missing artifact error: {err}"
    );
}

#[test]
fn artifact_checksums_reject_declared_directory_artifacts() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_checksums_directory_contract");
    let dir = fixture.path().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir: {e}"));
    let directory_artifact = dir.join("artifact_dir");
    std::fs::create_dir_all(&directory_artifact).unwrap_or_else(|e| panic!("mkdir artifact: {e}"));

    let err = bijux_dna_runtime::recording::write_artifact_checksums_json(
        &dir,
        &[("artifact_dir".to_string(), directory_artifact)],
    )
    .err()
    .unwrap_or_else(|| panic!("expected directory artifact to fail"));

    assert!(
        err.to_string().contains("must be a regular file"),
        "unexpected directory artifact error: {err}"
    );
}

#[test]
fn tool_invocation_writer_rejects_stage_id_path_separators() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_tool_invocation_path_contract");
    let dir = fixture.path().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir: {e}"));

    let err = bijux_dna_runtime::recording::write_tool_invocation_json(
        &dir,
        "../fastq.trim_reads",
        &tool_invocation_fixture(),
    )
    .err()
    .unwrap_or_else(|| panic!("expected invalid tool invocation stage id"));

    assert!(err.to_string().contains("file stem"), "unexpected invocation path error: {err}");
}

#[test]
fn atomic_writer_preserves_existing_tmp_sibling_files() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_atomic_tmp_contract");
    let dir = fixture.path().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir: {e}"));
    let target = dir.join("result.json");
    let sibling = dir.join("result.tmp");
    std::fs::write(&sibling, b"keep-me").unwrap_or_else(|e| panic!("write sibling: {e}"));

    bijux_dna_runtime::recording::write_atomic_bytes(&target, b"{\"ok\":true}")
        .unwrap_or_else(|e| panic!("write target: {e}"));

    assert_eq!(std::fs::read(&sibling).unwrap_or_else(|e| panic!("read sibling: {e}")), b"keep-me");
    assert_eq!(
        std::fs::read(&target).unwrap_or_else(|e| panic!("read target: {e}")),
        b"{\"ok\":true}"
    );
}

fn tool_invocation_fixture() -> bijux_dna_core::metrics::ToolInvocationV1 {
    bijux_dna_core::metrics::ToolInvocationV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        contract_version: bijux_dna_core::contract::ContractVersion::v1(),
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
    }
}

#[test]
fn jsonl_append_creates_nested_parent_directories() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_jsonl_parent_contract");
    let dir = fixture.path().to_path_buf();
    let path = dir.join("run_artifacts").join("telemetry").join("events.jsonl");

    bijux_dna_runtime::recording::append_jsonl_line(&path, "{\"event\":\"started\"}")
        .unwrap_or_else(|e| panic!("append jsonl: {e}"));

    assert_eq!(
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read jsonl: {e}")),
        "{\"event\":\"started\"}\n"
    );
}

#[test]
fn run_artifact_envelope_has_required_keys() {
    let fixture = bijux_dna_testkit::tempdir_for("runtime_envelope_contract");
    let dir = fixture.path().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir: {e}"));
    let path = bijux_dna_runtime::recording::write_run_artifact_envelope(
        &dir,
        "bam.validate",
        bijux_dna_runtime::recording::StageResultStatus::Ok,
        "SUCCESS",
    )
    .unwrap_or_else(|e| panic!("write envelope: {e}"));
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read envelope: {e}"));
    let value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse envelope: {e}"));
    for key in [
        "schema_version",
        "stage_id",
        "status",
        "reason_code",
        "manifest_json",
        "metrics_json",
        "checksums",
        "provenance",
        "logs",
    ] {
        assert!(value.get(key).is_some(), "missing key {key}");
    }
}

#[path = "manifest_integrity/output_declarations.rs"]
mod output_declarations;

#[cfg(unix)]
#[test]
fn profile_and_lock_manifests_capture_one_source_observation() -> anyhow::Result<()> {
    use anyhow::ensure;
    use profile_lock_capture::{source_document, verify_reports, SourceWriter};

    if profile_lock_capture::write_child_request()? {
        return Ok(());
    }
    let fixture = bijux_dna_testkit::tempdir_for("runtime-profile-lock-observation");
    let captured = source_document("captured-a");
    let replacement = source_document("replacement-b");
    let ordinary = fixture.path().join("ordinary");
    std::fs::create_dir(&ordinary)?;
    let ordinary_source = ordinary.join("run_manifest.json");
    std::fs::write(&ordinary_source, &captured)?;
    bijux_dna_runtime::write_profile_and_lock_manifests(&ordinary_source)?;
    verify_reports(&ordinary, &captured, "captured-a")?;
    std::fs::write(&ordinary_source, &replacement)?;
    bijux_dna_runtime::write_profile_and_lock_manifests(&ordinary_source)?;
    verify_reports(&ordinary, &replacement, "replacement-b")?;

    let profile_before = std::fs::read(ordinary.join("profile_manifest.json"))?;
    let lock_before = std::fs::read(ordinary.join("run_manifest.lock.json"))?;
    for (bytes, context) in [(b"{malformed\n".as_slice(), "parse "), (b"\xff\xfe", "read ")] {
        std::fs::write(&ordinary_source, bytes)?;
        let error = bijux_dna_runtime::write_profile_and_lock_manifests(&ordinary_source)
            .err()
            .unwrap_or_else(|| panic!("malformed source must refuse before publication"));
        ensure!(
            error.to_string().starts_with(context)
                && error.to_string().contains("run_manifest.json"),
            "unexpected source failure context: {error:#}"
        );
        assert_eq!(std::fs::read(ordinary.join("profile_manifest.json"))?, profile_before);
        assert_eq!(std::fs::read(ordinary.join("run_manifest.lock.json"))?, lock_before);
    }
    eprintln!(
        "profile-lock controls: ordinary A, fresh B, malformed and non-UTF8 retained outputs"
    );

    let directory = fixture.path().join("profile-lock-observation");
    std::fs::create_dir(&directory)?;
    let source = directory.join("run_manifest.json");
    let mut writer = SourceWriter::start(&source, &captured, &replacement)?;
    let result = bijux_dna_runtime::write_profile_and_lock_manifests(&source);
    writer.finish()?;
    result?;
    assert_eq!(std::fs::read(&source)?, replacement);
    eprintln!("profile-lock custody: replacement B published before captured A descriptor EOF");
    verify_reports(&directory, &captured, "captured-a")?;
    Ok(())
}

#[cfg(unix)]
mod profile_lock_capture {
    use anyhow::{ensure, Context, Result};
    use std::io::Write;
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::time::{Duration, Instant};

    const CASE: &str = "mod_contracts_manifest_integrity_rs::profile_and_lock_manifests_capture_one_source_observation";
    const REQUEST_VARIABLE: &str = "BIJUX_RUNTIME_PROFILE_LOCK_WRITER_REQUEST";

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct WriterInputs {
        captured_sha256: String,
        replacement_sha256: String,
        executable_sha256: String,
        executable_mode: u32,
    }

    pub(super) fn source_document(pipeline_id: &str) -> Vec<u8> {
        format!(
            "{{\n  \"schema_version\": \"bijux.run_manifest.v3\",\n  \"pipeline_id\": \"{pipeline_id}\",\n  \"stages\": [],\n  \"tool_invocations\": []\n}}\n"
        )
        .into_bytes()
    }

    pub(super) fn verify_reports(
        directory: &Path,
        captured: &[u8],
        pipeline_id: &str,
    ) -> Result<()> {
        let profile: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("profile_manifest.json"))?)?;
        let lock: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("run_manifest.lock.json"))?)?;
        assert_eq!(profile["pipeline_id"], pipeline_id);
        assert_eq!(
            lock["run_manifest"]["sha256"],
            bijux_dna_infra::hash_bytes_sha256(captured),
            "profile-lock source hash must describe the same captured source as profile fields"
        );
        Ok(())
    }

    fn verify_executable(executable: &Path, inputs: &WriterInputs) -> Result<()> {
        ensure!(
            bijux_dna_infra::hash_file_sha256(executable)? == inputs.executable_sha256
                && std::fs::metadata(executable)?.permissions().mode() == inputs.executable_mode
                && inputs.executable_mode & 0o111 != 0,
            "profile writer executable identity changed"
        );
        Ok(())
    }

    pub(super) fn write_child_request() -> Result<bool> {
        let Some(request) = std::env::var_os(REQUEST_VARIABLE) else { return Ok(false) };
        ensure!(
            std::env::args().skip(1).collect::<Vec<_>>() == [CASE, "--exact", "--nocapture"],
            "profile writer must be its exact isolated case"
        );
        let request = PathBuf::from(request);
        ensure!(
            request.is_absolute() && std::fs::symlink_metadata(&request)?.is_file(),
            "profile writer request must be an absolute regular file"
        );
        let directory = request.parent().context("profile writer directory")?.canonicalize()?;
        ensure!(
            request == directory.join("writer-request.json")
                && directory.file_name() == Some(std::ffi::OsStr::new("profile-lock-observation")),
            "profile writer request must belong to its canonical fixture"
        );
        let inputs: WriterInputs = serde_json::from_slice(&std::fs::read(&request)?)?;
        let captured = std::fs::read(directory.join("captured.json"))?;
        let replacement = std::fs::read(directory.join("replacement.json"))?;
        ensure!(
            captured == source_document("captured-a")
                && replacement == source_document("replacement-b")
                && bijux_dna_infra::hash_bytes_sha256(&captured) == inputs.captured_sha256
                && bijux_dna_infra::hash_bytes_sha256(&replacement) == inputs.replacement_sha256,
            "profile writer source identity changed"
        );
        let source = directory.join("run_manifest.json");
        ensure!(
            std::fs::symlink_metadata(&source)?.file_type().is_fifo()
                && !directory.join("writer-ready.json").try_exists()?
                && !directory.join("writer-receipt.json").try_exists()?,
            "profile writer requires its FIFO and fresh custody receipts"
        );
        verify_executable(&std::env::current_exe()?, &inputs)?;
        // This watchdog belongs only to the exact one-case writer child. The parent has none.
        let _watchdog = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(3));
            std::process::exit(124);
        });
        bijux_dna_infra::atomic_create_json(&directory.join("writer-ready.json"), &inputs)?;
        let mut fifo = std::fs::OpenOptions::new().write(true).open(&source)?;
        fifo.write_all(&captured)?;
        std::fs::rename(directory.join("replacement.json"), &source)?;
        assert_eq!(std::fs::read(&source)?, replacement);
        bijux_dna_infra::atomic_create_json(&directory.join("writer-receipt.json"), &inputs)?;
        drop(fifo);
        Ok(true)
    }

    struct OwnedChild(Child);

    impl OwnedChild {
        fn finish(&mut self, timeout: Duration) -> Result<ExitStatus> {
            let deadline = Instant::now() + timeout;
            loop {
                if let Some(status) = self.0.try_wait()? {
                    return Ok(status);
                }
                if Instant::now() >= deadline {
                    let _ = self.0.kill();
                    let _ = self.0.wait();
                    anyhow::bail!("profile observation child exceeded its owned wait deadline");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }

    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    pub(super) struct SourceWriter {
        child: OwnedChild,
        directory: PathBuf,
        inputs: WriterInputs,
        executable: PathBuf,
    }

    impl SourceWriter {
        pub(super) fn start(source: &Path, captured: &[u8], replacement: &[u8]) -> Result<Self> {
            let directory = source.parent().context("profile source directory")?.canonicalize()?;
            ensure!(source == directory.join("run_manifest.json"), "profile source must be owned");
            std::fs::write(directory.join("captured.json"), captured)?;
            std::fs::write(directory.join("replacement.json"), replacement)?;
            let executable = std::env::current_exe()?.canonicalize()?;
            let inputs = WriterInputs {
                captured_sha256: bijux_dna_infra::hash_bytes_sha256(captured),
                replacement_sha256: bijux_dna_infra::hash_bytes_sha256(replacement),
                executable_sha256: bijux_dna_infra::hash_file_sha256(&executable)?,
                executable_mode: std::fs::metadata(&executable)?.permissions().mode(),
            };
            let request = directory.join("writer-request.json");
            bijux_dna_infra::atomic_create_json(&request, &inputs)?;
            let mkfifo = std::env::split_paths(&std::env::var_os("PATH").context("mkfifo PATH")?)
                .map(|entry| entry.join("mkfifo"))
                .find(|path| path.is_file())
                .context("native mkfifo provider unavailable")?
                .canonicalize()?;
            let provider_sha256 = bijux_dna_infra::hash_file_sha256(&mkfifo)?;
            let provider_mode = std::fs::metadata(&mkfifo)?.permissions().mode();
            ensure!(provider_mode & 0o111 != 0, "mkfifo must be executable");
            let mut setup = OwnedChild(Command::new(&mkfifo).arg(source).spawn()?);
            ensure!(setup.finish(Duration::from_secs(1))?.success(), "mkfifo setup failed");
            ensure!(
                bijux_dna_infra::hash_file_sha256(&mkfifo)? == provider_sha256
                    && std::fs::metadata(&mkfifo)?.permissions().mode() == provider_mode,
                "mkfifo provider identity changed"
            );
            bijux_dna_infra::atomic_create_json(
                &directory.join("mkfifo-provider.json"),
                &serde_json::json!({"path":mkfifo,"sha256":provider_sha256,"mode":provider_mode}),
            )?;
            let log = std::fs::File::create(directory.join("writer.log"))?;
            let mut command = Command::new(std::env::current_exe()?);
            command
                .args([CASE, "--exact", "--nocapture"])
                .env(REQUEST_VARIABLE, &request)
                .stdin(Stdio::null())
                .stdout(log.try_clone()?)
                .stderr(log);
            // The writer inherits the owned parent group so an external cutoff covers both PIDs.
            let child = OwnedChild(command.spawn()?);
            let mut writer = Self { child, directory, inputs, executable };
            let deadline = Instant::now() + Duration::from_secs(1);
            while !writer.directory.join("writer-ready.json").try_exists()? {
                ensure!(writer.child.0.try_wait()?.is_none(), "profile writer exited before ready");
                ensure!(Instant::now() < deadline, "profile writer readiness deadline");
                std::thread::sleep(Duration::from_millis(5));
            }
            let ready: WriterInputs = serde_json::from_slice(&std::fs::read(
                writer.directory.join("writer-ready.json"),
            )?)?;
            assert_eq!(ready, writer.inputs);
            Ok(writer)
        }

        pub(super) fn finish(&mut self) -> Result<()> {
            ensure!(self.child.finish(Duration::from_secs(1))?.success(), "profile writer failed");
            let receipt: WriterInputs = serde_json::from_slice(&std::fs::read(
                self.directory.join("writer-receipt.json"),
            )?)?;
            assert_eq!(receipt, self.inputs);
            verify_executable(&self.executable, &self.inputs)
        }
    }
}
