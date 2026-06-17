#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_output_contract_tests_report_governs_all_retained_bindings() {
    let payload = run_cli_json(&["bench", "readiness", "render-output-contract-tests", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.output_contract_tests.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/output-contract-tests.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(140));
    assert_eq!(payload.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(140));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));

    let proof_surface_counts = payload
        .get("output_proof_surface_counts")
        .and_then(serde_json::Value::as_object)
        .expect("proof surface counts");
    assert_eq!(
        proof_surface_counts.get("shared_stage_smoke").and_then(serde_json::Value::as_u64),
        Some(115)
    );
    assert_eq!(
        proof_surface_counts.get("direct_tool_smoke").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        proof_surface_counts
            .get("direct_declared_output_proof")
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 140);

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_declared_output_proof")
            && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.haplogroups")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("yleaf")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_declared_output_proof")
            && row
                .get("observed_normalized_metric_ids")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some("haplogroups")))
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_tool_smoke")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}

#[test]
fn bench_readiness_output_contract_tests_write_governed_json_file() {
    let output = run_cli(&["bench", "readiness", "render-output-contract-tests"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/tools/output-contract-tests.json");

    let report_path = repo_root.join(rendered_path.trim());
    assert!(report_path.is_file(), "output contract report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.output_contract_tests.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(140));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("mapdamage2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.bias_mitigation")
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_declared_output_proof")
        }),
        "report file must retain governed BAM downstream direct-proof coverage"
    );
}
