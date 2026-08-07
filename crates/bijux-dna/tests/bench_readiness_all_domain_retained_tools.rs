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
fn bench_readiness_all_domain_retained_tools_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-retained-tools", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.retained_tool_inventory.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/retained-tool-inventory.tsv")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let row_count = u64::try_from(rows.len()).expect("row count fits u64");
    assert!(row_count > 0);
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(
        payload.get("active_matrix_tool_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    let benchmark_ready_tool_count =
        support::json_u64(&payload, "benchmark_ready_tool_count").expect("ready tool count");
    let not_benchmark_ready_only_tool_count =
        support::json_u64(&payload, "not_benchmark_ready_only_tool_count")
            .expect("not-ready-only tool count");
    assert_eq!(benchmark_ready_tool_count + not_benchmark_ready_only_tool_count, row_count);
    assert!(not_benchmark_ready_only_tool_count >= 4);

    let domain_counts = payload
        .get("domain_tool_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain counts");
    assert!(["fastq", "bam", "vcf"].iter().all(|domain| {
        domain_counts
            .get(*domain)
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0)
    }));
    for external_tool_id in ["angsd", "fastuniq", "impute5", "ngsbriggs"] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(external_tool_id)
                && row
                    .get("benchmark_statuses")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|statuses| {
                        statuses.iter().any(|status| status.as_str() == Some("not_benchmark_ready"))
                    })
        }));
    }

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
                == Some(&vec![
                    serde_json::Value::String("bam".to_string()),
                    serde_json::Value::String("vcf".to_string()),
                ])
            && row.get("active_stage_count").and_then(serde_json::Value::as_u64) == Some(12)
            && row.get("benchmark_ready_stage_count").and_then(serde_json::Value::as_u64)
                == Some(12)
            && row.get("active_binding_count").and_then(serde_json::Value::as_u64) == Some(12)
            && row.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64)
                == Some(12)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
    }));

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("domains").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("vcf".to_string())])
            && row.get("active_stage_count").and_then(serde_json::Value::as_u64) == Some(2)
            && row.get("benchmark_ready_stage_count").and_then(serde_json::Value::as_u64) == Some(2)
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eigensoft")
            && row.get("active_binding_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64)
                == Some(1)
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit")
            && row.get("domains").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("fastq".to_string())])
            && row.get("active_stage_count").and_then(serde_json::Value::as_u64) == Some(7)
            && row.get("benchmark_ready_stage_count").and_then(serde_json::Value::as_u64) == Some(7)
            && row.get("active_binding_count").and_then(serde_json::Value::as_u64) == Some(7)
            && row.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64)
                == Some(7)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("star")
            && row.get("active_stage_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("fastq.index_reference".to_string())])
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
            && row.get("active_stage_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("fastq.index_reference".to_string())])
    }));
}
