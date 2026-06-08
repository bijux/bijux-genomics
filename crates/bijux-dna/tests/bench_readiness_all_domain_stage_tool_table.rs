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
fn bench_readiness_all_domain_stage_tool_table_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-stage-tool-table", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_stage_tool_table.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domain-stage-tool-table.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(145));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(124)
    );
    assert_eq!(
        payload.get("benchmark_ready_unique_binding_count").and_then(serde_json::Value::as_u64),
        Some(124)
    );

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(74));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(22));

    let ready_domain_counts = payload
        .get("benchmark_ready_domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("ready domain counts");
    assert_eq!(ready_domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(ready_domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(ready_domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(12));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 145);

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("corpus-02-edna-mini")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("database_artifact_id+taxonomy_database_root")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("fastq.adapter.screen_taxonomy")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("fastq.parser.screen_taxonomy")
            && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                == Some("benchmark_ready")
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("schmutzi")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("corpus-01-adna-bam-mini")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("reference_fasta+reference_panel")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("bam.adapter.contamination")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("bam.parser.contamination")
            && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                == Some("benchmark_ready")
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("vcf_production_regression")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str) == Some("bam_bundle")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("vcf.adapter.calling")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("vcf.parser.call_summary")
            && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                == Some("benchmark_ready")
    }));
}
