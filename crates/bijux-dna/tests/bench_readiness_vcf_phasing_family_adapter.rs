#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(command_name: &str) -> serde_json::Value {
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
        .args(["bench", "readiness", command_name, "--json"])
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
fn bench_readiness_vcf_shapeit5_adapter_reports_governed_row() {
    let payload = run_cli_json("render-vcf-shapeit5-adapter");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_shapeit5_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("shapeit5"));
    assert_eq!(
        payload.get("tool_status").and_then(serde_json::Value::as_str),
        Some("experimental")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/shapeit5.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("indexed_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 1);
    let row = rows.first().expect("shapeit5 row");
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.phasing"));
    assert_eq!(
        row.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("benchmark_ready")
    );
    assert_eq!(
        row.get("panel_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_mini")
    );
    assert_eq!(
        row.get("map_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_chr_map")
    );
    assert_eq!(
        row.get("parser_output_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("phasing_qc".to_string()),
            serde_json::Value::String("phasing_manifest".to_string()),
        ])
    );

    let argv = row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("shapeit5 argv");
    for needle in ["shapeit5", "phase_common", "--reference", "--map", "--output"] {
        assert!(
            argv.iter().filter_map(serde_json::Value::as_str).any(|part| part.contains(needle)),
            "shapeit5 argv must retain `{needle}`"
        );
    }
}

#[test]
fn bench_readiness_vcf_eagle_adapter_reports_governed_row() {
    let payload = run_cli_json("render-vcf-eagle-adapter");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_eagle_adapter.v1")
    );
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("eagle"));
    assert_eq!(
        payload.get("tool_status").and_then(serde_json::Value::as_str),
        Some("experimental")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/eagle.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("indexed_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let row = payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .expect("eagle row");
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.phasing"));
    assert_eq!(
        row.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("not_benchmark_ready")
    );

    let argv = row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("eagle argv");
    for needle in ["eagle", "--vcfTarget", "--vcfRef", "--geneticMapFile", "--outPrefix"] {
        assert!(
            argv.iter().filter_map(serde_json::Value::as_str).any(|part| part.contains(needle)),
            "eagle argv must retain `{needle}`"
        );
    }
}

#[test]
fn bench_readiness_vcf_beagle_adapter_reports_governed_row() {
    let payload = run_cli_json("render-vcf-beagle-adapter");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_beagle_adapter.v1")
    );
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("beagle"));
    assert_eq!(
        payload.get("tool_status").and_then(serde_json::Value::as_str),
        Some("experimental")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/beagle.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("indexed_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let row = payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .expect("beagle row");
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.phasing"));
    assert_eq!(
        row.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("not_benchmark_ready")
    );

    let argv = row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("beagle argv");
    for needle in ["beagle", "gt=", "ref=", "map=", "out="] {
        assert!(
            argv.iter().filter_map(serde_json::Value::as_str).any(|part| part.contains(needle)),
            "beagle argv must retain `{needle}`"
        );
    }
}
