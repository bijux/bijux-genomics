#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_completion_check_writes_governed_fixture_tree() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-all-domain-completion-check"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/bench-readiness/completion-check-all-domains.json"
    );

    let report_path = repo_root.join("target/bench-readiness/completion-check-all-domains.json");
    assert!(report_path.is_file(), "expected completion-check report");
    let report_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read completion-check report"))
            .expect("parse completion-check report");

    let fixture_root =
        repo_root.join("target/bench-readiness/completion-check-all-domains-fixture");
    assert!(fixture_root.is_dir(), "expected completion-check fixture root");

    let seeded_mutations = report_json
        .get("seeded_mutations")
        .and_then(serde_json::Value::as_array)
        .expect("seeded mutations");
    let evidence_path = |mutation_id: &str| -> String {
        seeded_mutations
            .iter()
            .find(|mutation| {
                mutation.get("mutation_id").and_then(serde_json::Value::as_str) == Some(mutation_id)
            })
            .and_then(|mutation| mutation.get("evidence_path"))
            .and_then(serde_json::Value::as_str)
            .expect("seeded evidence path")
            .to_string()
    };

    let missing_manifest = repo_root.join(evidence_path("missing_manifest"));
    assert!(!missing_manifest.exists(), "seeded VCF manifest must stay absent");

    let empty_command = repo_root.join(evidence_path("required_file_empty"));
    assert!(empty_command.is_file(), "expected seeded command script");
    assert_eq!(
        fs::metadata(&empty_command).expect("stat empty command").len(),
        0,
        "seeded command script must stay empty"
    );

    let missing_normalized_metric = repo_root.join(evidence_path("missing_normalized_metrics"));
    assert!(!missing_normalized_metric.exists(), "seeded normalized metrics file must stay absent");

    let execution_manifest = repo_root.join(evidence_path("execution_not_successful"));
    let execution_manifest_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&execution_manifest).expect("read execution manifest"))
            .expect("parse execution manifest");
    assert_eq!(
        execution_manifest_json
            .get("runtime")
            .and_then(|value| value.get("exit_code"))
            .and_then(serde_json::Value::as_i64),
        Some(23)
    );
    assert_eq!(
        execution_manifest_json
            .get("runtime")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("failed")
    );

    let missing_declared_output = repo_root.join(evidence_path("missing_declared_output"));
    assert!(!missing_declared_output.exists(), "seeded declared output file must stay absent");
}
