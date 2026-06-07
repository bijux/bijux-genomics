#![allow(clippy::expect_used)]

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
fn bench_readiness_bam_report_map_reports_governed_stage_sections() {
    let payload = run_cli_json(&["bench", "readiness", "render-bam-report-map", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_report_map.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("bam"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam-report-map.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("section_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("summary_table_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload
            .get("section_counts")
            .and_then(|value| value.get("alignment_intake"))
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        payload
            .get("section_counts")
            .and_then(|value| value.get("alignment_refinement"))
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 24);

    let align = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align"))
        .expect("align row");
    assert_eq!(align.get("anchor_tool_id").and_then(serde_json::Value::as_str), Some("bwa"));
    assert_eq!(
        align.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("alignment_intake")
    );
    assert_eq!(
        align.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("alignment_baseline")
    );

    let contamination = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
        })
        .expect("contamination row");
    assert_eq!(
        contamination.get("anchor_tool_id").and_then(serde_json::Value::as_str),
        Some("schmutzi")
    );
    assert_eq!(
        contamination.get("workflow_branch_id").and_then(serde_json::Value::as_str),
        Some("ancient_dna_authenticity")
    );
    assert_eq!(
        contamination.get("criticality").and_then(serde_json::Value::as_str),
        Some("essential")
    );

    let sex = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex"))
        .expect("sex row");
    assert_eq!(sex.get("anchor_tool_id").and_then(serde_json::Value::as_str), Some("rxy"));
    assert_eq!(
        sex.get("scientific_context_required")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["chromosome_system", "minimum_y_sites"])
    );

    let recalibration = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.recalibration")
        })
        .expect("recalibration row");
    assert_eq!(
        recalibration.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("downstream_readiness")
    );
    assert_eq!(
        recalibration.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("variant_readiness")
    );
    assert_eq!(
        recalibration.get("criticality").and_then(serde_json::Value::as_str),
        Some("essential")
    );
}
