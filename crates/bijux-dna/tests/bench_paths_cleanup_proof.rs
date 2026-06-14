#![allow(clippy::expect_used)]

use std::path::Path;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, content).expect("write text");
}

fn copy_file(source: &Path, destination: &Path) {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).expect("create destination parent");
    }
    std::fs::copy(source, destination).expect("copy file");
}

fn init_repo(root: &Path) {
    let output =
        Command::new("git").arg("-C").arg(root).args(["init", "-q"]).output().expect("git init");
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["config", "user.email", "benchmarks@example.test"])
        .output()
        .expect("git config user.email");
    assert!(output.status.success());
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["config", "user.name", "benchmarks"])
        .output()
        .expect("git config user.name");
    assert!(output.status.success());
}

fn stage_all(root: &Path) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["add", "benchmarks", "tests"])
        .output()
        .expect("git add");
    assert!(output.status.success(), "git add failed: {}", String::from_utf8_lossy(&output.stderr));
}

fn write_benchmark_root(root: &Path) {
    write_text(&root.join("benchmarks/README.md"), "# Benchmarks\n");
    write_text(&root.join("benchmarks/configs/README.md"), "# Benchmark Configs\n");
    write_text(&root.join("benchmarks/schemas/README.md"), "# Benchmark Schemas\n");
    write_text(&root.join("benchmarks/tests/README.md"), "# Benchmark Tests\n");
    write_text(&root.join("benchmarks/readiness/README.md"), "# Benchmark Readiness\n");
    write_text(&root.join("benchmarks/readiness/local-ready/README.md"), "# Local-Ready Proof\n");
    write_text(
        &root.join("benchmarks/readiness/all-domain-schema-validation.json"),
        "{\n  \"ok\": true\n}\n",
    );
    write_text(
        &root.join("benchmarks/readiness/all-domain-stage-tool-table.tsv"),
        "stage_id\ttool_id\nfastq.validate_reads\tfastp\n",
    );
    write_text(&root.join("benchmarks/tests/fixtures/.gitkeep"), "");
    write_text(&root.join("tests/README.md"), "# Root Tests\n");
    std::fs::create_dir_all(root.join("tests")).expect("create tests root");
    #[cfg(unix)]
    std::os::unix::fs::symlink("../benchmarks/tests/fixtures", root.join("tests/fixtures"))
        .expect("symlink tests fixtures");
}

fn seed_disposable_root(path: &Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create disposable root parent");
    }
    std::fs::write(path, "disposable\n").expect("write disposable sentinel");
}

fn disposable_root_path(
    root: &Path,
    root_name: &str,
    leaf_group: &str,
    leaf_name: &str,
) -> std::path::PathBuf {
    root.join(root_name).join(leaf_group).join(leaf_name)
}

#[test]
fn bench_paths_cleanup_proof_reports_disposable_root_deletion() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let home = tempfile::tempdir().expect("tempdir");
    let source_repo_root = support::repo_root().expect("source repo root");
    let repo_root = tempfile::tempdir().expect("repo tempdir");
    init_repo(repo_root.path());
    write_benchmark_root(repo_root.path());
    copy_file(
        &source_repo_root.join("configs/runtime/platforms.toml"),
        &repo_root.path().join("configs/runtime/platforms.toml"),
    );
    copy_file(
        &source_repo_root.join("configs/ci/tools/images.toml"),
        &repo_root.path().join("configs/ci/tools/images.toml"),
    );
    stage_all(repo_root.path());
    write_text(
        &repo_root.path().join("benchmarks/readiness/untracked-snapshot.json"),
        "{\n  \"ok\": false\n}\n",
    );
    seed_disposable_root(&disposable_root_path(
        repo_root.path(),
        "target",
        "goal-315-json",
        "sentinel.txt",
    ));
    seed_disposable_root(&disposable_root_path(
        repo_root.path(),
        "artifacts",
        "goal-315-json",
        "sentinel.txt",
    ));
    seed_disposable_root(&disposable_root_path(
        repo_root.path(),
        "runs",
        "goal-315-json",
        "sentinel.txt",
    ));

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(repo_root.path())
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "paths", "prove-disposable-root-cleanup", "--json"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.disposable_root_cleanup_proof.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/path-cleanup/DELETE_DISPOSABLE_ROOTS_SAFE.json")
    );
    assert_eq!(
        payload.get("validator_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/benchmark-paths-validation.json")
    );
    assert_eq!(payload.get("validator_ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("validator_violation_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("validator_readiness_json_snapshot_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("validator_readiness_tsv_snapshot_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("deleted_root_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        payload.get("already_absent_root_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    let deleted_roots = payload
        .get("deleted_roots")
        .and_then(serde_json::Value::as_array)
        .expect("deleted roots array");
    assert_eq!(deleted_roots.len(), 3);
    assert!(deleted_roots.iter().all(|value| {
        value.get("exists_after").and_then(serde_json::Value::as_bool) == Some(false)
    }));
}
