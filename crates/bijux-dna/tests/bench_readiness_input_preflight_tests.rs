#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
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
        .args(["bench", "readiness", "render-input-preflight-tests", "--json"])
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
fn bench_readiness_input_preflight_tests_report_governed_retained_tool_coverage() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.input_preflight_tests.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/input-preflight-tests.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(258));
    assert_eq!(payload.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(258));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("retained_tool_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(payload.get("covered_tool_count").and_then(serde_json::Value::as_u64), Some(71));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(112));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(96));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(50));

    let missing_input_class_counts = payload
        .get("missing_input_class_counts")
        .and_then(serde_json::Value::as_object)
        .expect("missing input class counts");
    for required_class in ["fastq", "bam", "vcf", "reference", "database", "panel"] {
        assert!(
            missing_input_class_counts.contains_key(required_class),
            "governed report must retain coverage for {required_class}"
        );
    }

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 258);

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bwa")
            && row.get("missing_input_role").and_then(serde_json::Value::as_str)
                == Some("reference_index")
            && row.get("artifact_path").and_then(serde_json::Value::as_str)
                == Some("assets/reference/host/references/toy_host_reference")
            && row.get("contract_surface").and_then(serde_json::Value::as_str)
                == Some("runtime_validation")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.merge_pairs")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("adapterremoval")
            && row.get("missing_input_role").and_then(serde_json::Value::as_str) == Some("reads_r1")
            && row
                .get("artifact_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.ends_with("human_like_pe_merge_overlap_R1.fastq.gz"))
            && row.get("observed_error").and_then(serde_json::Value::as_str).is_some_and(|error| {
                error.contains("missing required input reads_r1")
                    && error.contains("reads_r1.fastq.gz")
            })
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
            && row.get("missing_input_role").and_then(serde_json::Value::as_str)
                == Some("reads_r2")
            && row
                .get("artifact_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.ends_with("mock_community_reads_R2.fastq"))
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("missing_input_role").and_then(serde_json::Value::as_str)
                == Some("vcf_index")
            && row.get("contract_surface").and_then(serde_json::Value::as_str)
                == Some("adapter_contract")
            && row
                .get("artifact_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.ends_with("phasing_input.vcf.gz.tbi"))
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}

#[test]
fn bench_readiness_input_preflight_tests_write_governed_json_file() {
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
        .args(["bench", "readiness", "render-input-preflight-tests"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/tools/input-preflight-tests.json");

    let report_path = repo_root.join(rendered_path.trim());
    assert!(report_path.is_file(), "input preflight report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.input_preflight_tests.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(258));
    assert_eq!(payload.get("covered_tool_count").and_then(serde_json::Value::as_u64), Some(71));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fixture_contract")
                && row.get("missing_input_role").and_then(serde_json::Value::as_str)
                    == Some("sites_bed")
                && row.get("contract_surface").and_then(serde_json::Value::as_str)
                    == Some("fixture_support")
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
        }),
        "report file must retain the governed fixture-support probe row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("verifybamid2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.contamination")
                && row.get("missing_input_role").and_then(serde_json::Value::as_str)
                    == Some("reference_panel")
                && row
                    .get("artifact_path")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|path| path.ends_with("adna_contamination_panel.dat"))
        }),
        "report file must retain governed contamination-panel coverage"
    );
}
