#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn dev_crates_parser_effect_audit_writes_the_governed_audit_report() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("parser-no-execution.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("parser-no-execution")
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
        Some("bijux.crates.parser_no_execution.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("audited_surface_count").and_then(serde_json::Value::as_u64), Some(3));

    let surfaces =
        payload.get("surfaces").and_then(serde_json::Value::as_array).expect("surfaces array");
    for crate_name in ["bijux-dna-domain-bam", "bijux-dna-domain-fastq", "bijux-dna-domain-vcf"] {
        let report = surfaces
            .iter()
            .find(|surface| {
                surface.get("crate_name").and_then(serde_json::Value::as_str) == Some(crate_name)
            })
            .unwrap_or_else(|| panic!("missing parser surface for {crate_name}"));
        assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
        assert!(
            report
                .get("parser_entrypoints")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|rows| !rows.is_empty()),
            "{crate_name} must report parser entrypoints"
        );
        for key in [
            "input_mutation_refs",
            "process_execution_refs",
            "container_execution_refs",
            "slurm_execution_refs",
        ] {
            let rows = report.get(key).and_then(serde_json::Value::as_array).expect(key);
            assert!(rows.is_empty(), "{crate_name} must have no {key}");
        }
    }

    let fastq_surface = surfaces
        .iter()
        .find(|surface| {
            surface.get("crate_name").and_then(serde_json::Value::as_str)
                == Some("bijux-dna-domain-fastq")
        })
        .expect("fastq surface");
    let excluded = fastq_surface
        .get("excluded_governance_paths")
        .and_then(serde_json::Value::as_array)
        .expect("excluded governance paths");
    assert!(
        excluded.iter().any(|row| {
            row.get("path").and_then(serde_json::Value::as_str)
                == Some("crates/bijux-dna-domain-fastq/src/observer/parse/raw_parser_contract.rs")
        }),
        "FASTQ parser audit must declare the governance-only raw parser contract exclusion"
    );

    assert!(out.is_file(), "audit must write the governed report file");
}
