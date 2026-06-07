#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_validate_all_domain_schemas_writes_governed_summary_file() {
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
        .args([
            "bench",
            "validate-schemas",
            "--schema-root",
            "benchmarks/schemas",
            "--domain",
            "fastq,bam,vcf",
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domain-schema-validation.json");

    let report_path = repo_root.join(rendered_path.trim());
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&report_path).expect("read validation report"),
    )
    .expect("parse validation report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_schema_validation.v1")
    );
    assert_eq!(
        report.get("schema_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas")
    );
    assert_eq!(report.get("ok"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(report.get("domain_count").and_then(serde_json::Value::as_u64), Some(3));
}
