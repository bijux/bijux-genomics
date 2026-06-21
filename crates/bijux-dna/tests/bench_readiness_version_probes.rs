#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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
        .args(args)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_version_probes_report_governed_probe_contracts() {
    let payload = run_cli_json(&["bench", "readiness", "render-version-probes", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.version_probes.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/version-probes.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(payload.get("ready_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("unavailable_count").and_then(serde_json::Value::as_u64), Some(2));

    let parser_kind_counts = payload
        .get("parser_kind_counts")
        .and_then(serde_json::Value::as_object)
        .expect("parser kind counts");
    assert_eq!(
        parser_kind_counts.get("first_dotted_numeric_token").and_then(serde_json::Value::as_u64),
        Some(69)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 71);

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str) == Some("ready")
            && row.get("version_cmd").and_then(serde_json::Value::as_str)
                == Some("bijux-dna --version")
            && row.get("expected_bin").and_then(serde_json::Value::as_str) == Some("bijux-dna")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str) == Some("ready")
            && row.get("expected_version_regex").and_then(serde_json::Value::as_str)
                == Some("bcftools [0-9]+([.][0-9]+)?")
            && row.get("registry_paths").and_then(serde_json::Value::as_array).is_some_and(
                |paths| {
                    paths.iter().any(|path| {
                        path.as_str() == Some("configs/ci/registry/tool_registry_vcf.toml")
                    })
                },
            )
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqfu")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str) == Some("ready")
            && row.get("registry_paths").and_then(serde_json::Value::as_array).is_some_and(
                |paths| {
                    paths.iter().any(|path| {
                        path.as_str()
                            == Some("configs/ci/registry/tool_registry_experimental.toml")
                    })
                },
            )
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str)
                == Some("unavailable_with_reason")
            && row
                .get("unavailable_reason")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|reason| reason.contains("external container source"))
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str)
                == Some("unavailable_with_reason")
            && row
                .get("unavailable_reason")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|reason| reason.contains("planned container source"))
    }));
}
