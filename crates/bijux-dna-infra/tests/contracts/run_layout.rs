use std::path::{Path, PathBuf};
use std::time::Duration;

#[test]
fn lock_acquisition_times_out() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let lock_path = dir.path().join("lockfile");
    let _lock = bijux_dna_infra::FileLock::acquire(&lock_path, Duration::from_millis(50))?;
    let err = bijux_dna_infra::FileLock::acquire(&lock_path, Duration::from_millis(50))
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected lock timeout"))?;
    assert_eq!(err.kind, bijux_dna_infra::IoErrorKind::LockTimeout);
    Ok(())
}

#[test]
fn file_lock_creates_parent_directory() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let lock_path = dir.path().join("nested").join("lockfile");

    let _lock = bijux_dna_infra::FileLock::acquire(&lock_path, Duration::from_millis(50))?;

    assert!(lock_path.exists());
    Ok(())
}

#[test]
fn run_layout_is_stable() {
    let base = Path::new("/tmp/bijux");
    let layout = bijux_dna_infra::run_layout_paths(base, "run-123");
    assert_eq!(layout.run_dir, base.join("runs").join("run-123"));
    assert_eq!(layout.artifacts_dir, base.join("runs").join("run-123").join("artifacts"));
    assert_eq!(layout.logs_dir, base.join("runs").join("run-123").join("logs"));
    assert_eq!(layout.tmp_dir, base.join("runs").join("run-123").join("tmp"));
}

#[test]
fn run_layout_paths_confine_run_id() {
    let base = Path::new("/tmp/bijux");
    let layout = bijux_dna_infra::run_layout_paths(base, "../run/123");
    assert_eq!(layout.run_dir, base.join("runs").join("run_123"));
    assert_eq!(layout.artifacts_dir, base.join("runs").join("run_123").join("artifacts"));
}

#[test]
fn run_stage_dir_confines_stage_and_tool() {
    let base = Path::new("/tmp/bijux");
    let path = bijux_dna_infra::run_stage_dir(base, "run-1", "../stage/a", "/tool/b");
    assert_eq!(path, base.join("runs").join("run-1").join("stage_a").join("tool_b"));
}

#[test]
fn run_layout_contract_is_enforced() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let layout = bijux_dna_infra::run_layout_paths(dir.path(), "run-1");
    let _lock = bijux_dna_infra::lock_run(&layout, Duration::from_millis(50))?;
    let marker = bijux_dna_infra::publish_run(&layout, "run-1")?;
    assert!(marker.ends_with(bijux_dna_infra::RUN_LAYOUT_CONTRACT.publish_marker));
    Ok(())
}

#[test]
fn pipeline_run_dir_contract_is_stable() {
    let base = Path::new("/tmp/bijux");
    let path = bijux_dna_infra::pipeline_run_dir(
        base,
        "fastq-to-fastq__default__v1",
        "sample-1",
        "run-abc",
    );
    assert_eq!(
        path,
        PathBuf::from("/tmp/bijux")
            .join("fastq-to-fastq__default__v1")
            .join("sample-1")
            .join("run-abc")
    );
    assert_eq!(bijux_dna_infra::PIPELINE_RUN_DIR_TEMPLATE, "{pipeline_id}/{sample_id}/{run_id}");
}

#[test]
fn pipeline_run_dir_confines_dynamic_segments() {
    let base = Path::new("/tmp/bijux");
    let path = bijux_dna_infra::pipeline_run_dir(base, "/absolute/pipeline", "../sample", "run/a");
    assert_eq!(
        path,
        PathBuf::from("/tmp/bijux").join("absolute_pipeline").join("sample").join("run_a")
    );
}

#[test]
fn configs_file_confines_relative_inputs() {
    let root = Path::new("/tmp/bijux");
    assert_eq!(
        bijux_dna_infra::configs_file(root, "/runtime/platforms.toml"),
        root.join("configs").join("runtime").join("platforms.toml")
    );
    assert_eq!(
        bijux_dna_infra::configs_file(root, "../secrets.toml"),
        root.join("configs").join("secrets.toml")
    );
}

#[test]
fn bench_base_dir_confines_stage_and_sample() {
    let out = Path::new("/tmp/bijux");
    assert_eq!(
        bijux_dna_infra::bench_base_dir(out, "../stage", "/sample/one"),
        out.join("bench").join("stage").join("sample_one")
    );
}

fn attempt_identity() -> bijux_dna_infra::AttemptLayoutIdentity {
    bijux_dna_infra::AttemptLayoutIdentity {
        run_directory: "run-2042-fastq-profile-read-lengths".to_string(),
        tool_id: "seqkit".to_string(),
        campaign_id: "all-domains-20260804".to_string(),
        attempt_id: "attempt-001".to_string(),
    }
}

#[test]
fn governed_attempt_layout_uses_permanent_identity_and_required_directories() -> anyhow::Result<()>
{
    let temporary = bijux_dna_infra::temp_dir("bijux-attempt-layout")?;
    let benchmark_root = temporary.path().join("benchmarks");
    let identity = attempt_identity();

    let layout = bijux_dna_infra::ensure_attempt_layout(
        &benchmark_root,
        bijux_dna_infra::AttemptEvidenceOwner::Benchmarks,
        &identity,
    )?;

    assert_eq!(
        layout.attempt_dir.strip_prefix(&benchmark_root)?,
        Path::new(
            "run-2042-fastq-profile-read-lengths/seqkit/attempts/all-domains-20260804/attempt-001"
        )
    );
    for directory in [
        &layout.manifests_dir,
        &layout.logs_dir,
        &layout.outputs_dir,
        &layout.accounting_dir,
        &layout.validation_dir,
        &layout.science_dir,
        &layout.receipts_dir,
    ] {
        assert!(directory.is_dir());
    }
    let manifest: bijux_dna_infra::AttemptLayoutManifestV1 =
        serde_json::from_slice(&std::fs::read(&layout.layout_manifest_path)?)?;
    assert_eq!(manifest.schema_version, bijux_dna_infra::ATTEMPT_LAYOUT_SCHEMA_V1);
    assert_eq!(manifest.owner, bijux_dna_infra::AttemptEvidenceOwner::Benchmarks);
    assert_eq!(manifest.attempt_id, "attempt-001");
    Ok(())
}

#[test]
fn governed_attempt_layout_recovery_is_idempotent_and_owner_locked() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("bijux-attempt-recovery")?;
    let benchmark_root = temporary.path().join("benchmarks");
    let identity = attempt_identity();
    let first = bijux_dna_infra::ensure_attempt_layout(
        &benchmark_root,
        bijux_dna_infra::AttemptEvidenceOwner::Benchmarks,
        &identity,
    )?;
    let manifest_bytes = std::fs::read(&first.layout_manifest_path)?;
    std::fs::write(first.outputs_dir.join("result.json"), b"{}")?;

    let recovered = bijux_dna_infra::ensure_attempt_layout(
        &benchmark_root,
        bijux_dna_infra::AttemptEvidenceOwner::Benchmarks,
        &identity,
    )?;
    assert_eq!(recovered, first);
    assert_eq!(std::fs::read(&recovered.layout_manifest_path)?, manifest_bytes);
    assert!(recovered.outputs_dir.join("result.json").is_file());
    assert!(bijux_dna_infra::ensure_attempt_layout(
        &benchmark_root,
        bijux_dna_infra::AttemptEvidenceOwner::Containers,
        &identity,
    )
    .is_err());
    Ok(())
}

#[test]
fn governed_attempt_layout_refuses_partial_or_unsafe_identity_reuse() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("bijux-attempt-refusal")?;
    let benchmark_root = temporary.path().join("benchmarks");
    let mut identity = attempt_identity();
    let layout = bijux_dna_infra::attempt_layout_paths(&benchmark_root, &identity)?;
    std::fs::create_dir_all(&layout.attempt_dir)?;
    std::fs::write(layout.attempt_dir.join("foreign.txt"), b"partial")?;
    assert!(bijux_dna_infra::ensure_attempt_layout(
        &benchmark_root,
        bijux_dna_infra::AttemptEvidenceOwner::Benchmarks,
        &identity,
    )
    .is_err());
    assert_eq!(std::fs::read(layout.attempt_dir.join("foreign.txt"))?, b"partial");

    identity.attempt_id = "../attempt-002".to_string();
    assert!(bijux_dna_infra::attempt_layout_paths(&benchmark_root, &identity).is_err());
    Ok(())
}

#[test]
fn governed_attempt_owner_roots_must_be_separate() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("bijux-attempt-owner-roots")?;
    let containers = temporary.path().join("containers");
    let benchmarks = temporary.path().join("benchmarks");
    bijux_dna_infra::validate_attempt_owner_roots(&containers, &benchmarks)?;
    assert!(bijux_dna_infra::validate_attempt_owner_roots(&containers, &containers).is_err());
    assert!(bijux_dna_infra::validate_attempt_owner_roots(
        &containers,
        &containers.join("benchmarks")
    )
    .is_err());
    assert!(bijux_dna_infra::validate_attempt_owner_roots(
        Path::new("relative-containers"),
        &benchmarks
    )
    .is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn governed_attempt_owner_roots_reject_symlinked_components() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let temporary = bijux_dna_infra::temp_dir("bijux-attempt-owner-symlink")?;
    let physical = temporary.path().join("physical");
    let alias = temporary.path().join("alias");
    std::fs::create_dir(&physical)?;
    symlink(&physical, &alias)?;

    let aliased_root = alias.join("attempts");
    let physical_root = physical.join("attempts");
    assert!(bijux_dna_infra::validate_attempt_owner_roots(&aliased_root, &physical_root).is_err());
    Ok(())
}

#[test]
fn governed_attempt_layout_schema_closes_owner_and_directory_shape() {
    let schema = bijux_dna_infra::attempt_layout_schema();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["schema_version"]["const"], "bijux.attempt_layout.v1");
    assert_eq!(schema["properties"]["attempt_id"]["pattern"], "^[A-Za-z0-9][A-Za-z0-9._-]*$");
    assert_eq!(schema["properties"]["directories"]["properties"]["receipts"]["const"], "receipts");
}
