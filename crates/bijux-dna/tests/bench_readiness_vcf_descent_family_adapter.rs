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
        .args(["bench", "readiness", "render-vcf-descent-family-adapter", "--json"])
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
fn bench_readiness_vcf_descent_family_adapter_reports_governed_rows() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_descent_family_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/descent-family.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 5);

    for (tool_id, stage_id, benchmark_status) in [
        ("plink2", "vcf.roh", "benchmark_ready"),
        ("germline", "vcf.ibd", "benchmark_ready"),
        ("ibdseq", "vcf.ibd", "not_benchmark_ready"),
        ("ibdhap", "vcf.ibd", "not_benchmark_ready"),
        ("ibdne", "vcf.demography", "benchmark_ready"),
    ] {
        let row = rows
            .iter()
            .find(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
            })
            .unwrap_or_else(|| panic!("missing governed row {tool_id}:{stage_id}"));
        assert_eq!(
            row.get("benchmark_status").and_then(serde_json::Value::as_str),
            Some(benchmark_status)
        );
        assert_eq!(
            row.get("missing_input_test_passed").and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }

    let roh = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
        })
        .expect("plink2 roh row");
    assert_eq!(
        roh.get("normalized_output_artifact_id").and_then(serde_json::Value::as_str),
        Some("roh_report")
    );
    let roh_argv = roh
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("roh argv");
    for needle in ["plink2", "--vcf", "--homozyg", "--out"] {
        assert!(
            roh_argv.iter().filter_map(serde_json::Value::as_str).any(|part| part.contains(needle)),
            "roh argv must retain `{needle}`"
        );
    }

    let germline = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("germline")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
        })
        .expect("germline row");
    let germline_argv = germline
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("germline argv");
    for needle in ["plink2 --vcf", "--make-bed", "germline", "-output"] {
        assert!(
            germline_argv
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|part| part.contains(needle)),
            "germline argv must retain `{needle}`"
        );
    }

    let ibdseq = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ibdseq")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
        })
        .expect("ibdseq row");
    let ibdseq_argv = ibdseq
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("ibdseq argv");
    assert!(
        ibdseq_argv
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|part| part.contains("ibdseq --vcf")),
        "ibdseq argv must retain the retained ibdseq invocation"
    );

    let ibdhap = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ibdhap")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
        })
        .expect("ibdhap row");
    let ibdhap_argv = ibdhap
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("ibdhap argv");
    assert!(
        ibdhap_argv
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|part| part.contains("ibdhap --vcf")),
        "ibdhap argv must retain the retained ibdhap invocation"
    );

    let demography = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ibdne")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.demography")
        })
        .expect("ibdne row");
    assert!(
        demography
            .get("input_ibd_segments_path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| path.ends_with("artifacts/input/ibd_segments.tsv")),
        "ibdne row must materialize the governed IBD segments input"
    );
    let demography_argv = demography
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("ibdne argv");
    assert!(
        demography_argv
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|part| part.contains("ibdne --ibd")),
        "ibdne argv must retain the retained ibdne invocation"
    );
}
