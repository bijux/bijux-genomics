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
fn bench_readiness_essential_pipeline_report_map_tracks_all_pipeline_outputs() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-essential-pipeline-report-map", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipeline_report_map.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/essential-pipeline-report-map.tsv")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(45));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(267));
    assert_eq!(payload.get("report_section_count").and_then(serde_json::Value::as_u64), Some(22));
    assert_eq!(payload.get("failure_column_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload
            .get("pipeline_row_counts")
            .and_then(|value| value.get("diploid-small-fastq-bam-vcf"))
            .and_then(serde_json::Value::as_u64),
        Some(45)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("vcf"))
            .and_then(serde_json::Value::as_u64),
        Some(115)
    );
    assert_eq!(
        payload
            .get("report_section_counts")
            .and_then(|value| value.get("quality_control"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 267);

    assert!(rows.iter().all(|row| {
        row.get("pipeline_id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.is_empty())
            && row
                .get("stage_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty())
            && row
                .get("tool_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty())
            && row
                .get("output_metric")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty())
            && row
                .get("report_section")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty())
            && row.get("failure_column").and_then(serde_json::Value::as_str)
                == Some("failure_reason")
    }));

    let core_call = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("core-germline-fastq-bam-vcf")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
                && row.get("output_metric").and_then(serde_json::Value::as_str)
                    == Some("called_vcf")
        })
        .expect("core vcf.call row");
    assert_eq!(core_call.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        core_call.get("report_section").and_then(serde_json::Value::as_str),
        Some("variant_calling")
    );

    let edna_taxonomy = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("edna-taxonomy-no-vcf")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
                && row.get("output_metric").and_then(serde_json::Value::as_str)
                    == Some("taxonomy_summary")
        })
        .expect("eDNA taxonomy summary row");
    assert_eq!(edna_taxonomy.get("tool_id").and_then(serde_json::Value::as_str), Some("kraken2"));
    assert_eq!(
        edna_taxonomy.get("report_section").and_then(serde_json::Value::as_str),
        Some("contamination_screening")
    );

    let relatedness_demography = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.demography")
                && row.get("output_metric").and_then(serde_json::Value::as_str)
                    == Some("demography_report")
        })
        .expect("demography report row");
    assert_eq!(
        relatedness_demography.get("tool_id").and_then(serde_json::Value::as_str),
        Some("ibdne")
    );
    assert_eq!(
        relatedness_demography.get("report_section").and_then(serde_json::Value::as_str),
        Some("demography")
    );
}
