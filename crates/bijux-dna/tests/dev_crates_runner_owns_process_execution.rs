#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn dev_crates_runner_owns_process_execution_writes_the_governed_audit_report() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("runner-owns-process-execution.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("runner-owns-process-execution")
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
        Some("bijux.crates.runner_owned_process_execution.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("audited_crate_count").and_then(serde_json::Value::as_u64), Some(9));

    let crates = payload.get("crates").and_then(serde_json::Value::as_array).expect("crates array");
    for crate_name in [
        "bijux-dna-analyze",
        "bijux-dna-bench-model",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-compiler",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-vcf",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
    ] {
        let report = crates
            .iter()
            .find(|crate_report| {
                crate_report.get("crate_name").and_then(serde_json::Value::as_str)
                    == Some(crate_name)
            })
            .unwrap_or_else(|| panic!("missing crate report for {crate_name}"));
        assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
        for key in ["process_execution_refs", "container_execution_refs", "slurm_execution_refs"] {
            let rows = report.get(key).and_then(serde_json::Value::as_array).expect(key);
            assert!(rows.is_empty(), "{crate_name} must have no {key}");
        }
    }

    let stages_vcf = crates
        .iter()
        .find(|crate_report| {
            crate_report.get("crate_name").and_then(serde_json::Value::as_str)
                == Some("bijux-dna-stages-vcf")
        })
        .expect("stages-vcf report");
    let execution_owner_dependencies = stages_vcf
        .get("execution_owner_dependencies")
        .and_then(serde_json::Value::as_array)
        .expect("execution_owner_dependencies");
    assert!(
        execution_owner_dependencies.iter().any(|row| {
            row.get("dependency").and_then(serde_json::Value::as_str) == Some("bijux-dna-runner")
                && row.get("section").and_then(serde_json::Value::as_str) == Some("dependencies")
        }),
        "stages-vcf must keep the runner execution-owner dependency explicit"
    );

    assert!(out.is_file(), "audit must write the governed report file");
}
