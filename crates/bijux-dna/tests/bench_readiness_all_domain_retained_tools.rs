#![allow(clippy::expect_used)]

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
fn bench_readiness_all_domain_retained_tools_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-retained-tools", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_retained_tools.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/retained-tools.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(64));
    assert_eq!(
        payload.get("active_matrix_tool_count").and_then(serde_json::Value::as_u64),
        Some(64)
    );
    assert_eq!(
        payload.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64),
        Some(64)
    );
    assert_eq!(payload.get("mixed_status_tool_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("not_benchmark_ready_only_tool_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );

    let domain_counts = payload
        .get("domain_tool_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(39));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(1));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 64);

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
            && row.get("domains").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("fastq".to_string())])
            && row.get("active_stage_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_ready_stage_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("active_binding_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64)
                == Some(1)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
            && row.get("active_stage_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("fastq.screen_taxonomy".to_string())])
            && row.get("benchmark_ready_stage_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("fastq.screen_taxonomy".to_string())])
    }));

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("domains").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("vcf".to_string())])
            && row.get("active_stage_count").and_then(serde_json::Value::as_u64) == Some(8)
            && row.get("benchmark_ready_stage_count").and_then(serde_json::Value::as_u64) == Some(8)
            && row.get("active_binding_count").and_then(serde_json::Value::as_u64) == Some(8)
            && row.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64)
                == Some(8)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
    }));

    assert!(
        rows.iter()
            .all(|row| row.get("tool_id").and_then(serde_json::Value::as_str) != Some("beagle")),
        "planned-only tools must be outside retained active scope"
    );
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit")
            && row.get("domains").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("fastq".to_string())])
            && row.get("active_stage_count").and_then(serde_json::Value::as_u64) == Some(5)
            && row.get("benchmark_ready_stage_count").and_then(serde_json::Value::as_u64) == Some(5)
            && row.get("active_binding_count").and_then(serde_json::Value::as_u64) == Some(5)
            && row.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64)
                == Some(5)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
    }));
    assert!(
        rows.iter().all(|row| {
            !matches!(
                row.get("tool_id").and_then(serde_json::Value::as_str),
                Some("star" | "bowtie2_build")
            )
        }),
        "retained tools must exclude not-benchmark-ready-only tools"
    );
}
