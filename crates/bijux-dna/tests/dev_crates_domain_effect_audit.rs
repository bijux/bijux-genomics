#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn dev_crates_domain_effect_audit_writes_the_governed_audit_report() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("domain-no-execution.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("domain-no-execution")
        .arg("--output")
        .arg(&out)
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
        serde_json::from_slice(&output.stdout).expect("stdout json payload");
    let expected_output_path = out.display().to_string();
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.crates.domain_no_execution.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("audited_crate_count").and_then(serde_json::Value::as_u64), Some(3));
    let crates = payload.get("crates").and_then(serde_json::Value::as_array).expect("crates array");
    for crate_name in ["bijux-dna-domain-bam", "bijux-dna-domain-fastq", "bijux-dna-domain-vcf"] {
        let report = crates
            .iter()
            .find(|crate_report| {
                crate_report.get("crate_name").and_then(serde_json::Value::as_str)
                    == Some(crate_name)
            })
            .unwrap_or_else(|| panic!("missing crate report for {crate_name}"));
        assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
        for key in [
            "forbidden_direct_dependencies",
            "process_execution_refs",
            "container_execution_refs",
            "slurm_execution_refs",
        ] {
            let rows = report.get(key).and_then(serde_json::Value::as_array).expect(key);
            assert!(rows.is_empty(), "{crate_name} must have no {key}");
        }
    }
    assert!(out.is_file(), "audit must write the governed report file");
}
