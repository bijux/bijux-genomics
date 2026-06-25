#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
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
fn bench_local_vcf_micro_smoke_subset_reports_one_governed_row_per_family() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-micro-smoke-subset", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_micro_smoke_subset.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json")
    );
    assert_eq!(payload.get("family_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("local_smoke_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("container_needed_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("unavailable_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 8);

    let family_ids = rows
        .iter()
        .filter_map(|row| row.get("family_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        family_ids,
        BTreeSet::from([
            "vcf.calling",
            "vcf.descent_and_demography",
            "vcf.imputation",
            "vcf.phasing",
            "vcf.population_structure",
            "vcf.quality_control",
            "vcf.reference_panel_preparation",
            "vcf.variant_curation",
        ])
    );

    let calling = rows
        .iter()
        .find(|row| row.get("family_id").and_then(serde_json::Value::as_str) == Some("vcf.calling"))
        .expect("vcf.calling family row");
    assert_eq!(
        calling.get("execution_status").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(
        calling.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.call")
    );
    assert_eq!(
        calling.get("representative_tool_id").and_then(serde_json::Value::as_str),
        Some("bcftools")
    );
    assert_eq!(calling.get("evidence_format").and_then(serde_json::Value::as_str), Some("json"));
    assert_eq!(
        calling.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );

    let phasing = rows
        .iter()
        .find(|row| row.get("family_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing"))
        .expect("vcf.phasing family row");
    assert_eq!(
        phasing.get("execution_status").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(
        phasing.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.phasing")
    );
    assert_eq!(
        phasing.get("representative_tool_id").and_then(serde_json::Value::as_str),
        Some("shapeit5")
    );
    assert!(phasing.get("smoke_command").and_then(serde_json::Value::as_str).is_some_and(
        |command| { command == "bijux-dna bench local run-vcf-phasing-smoke --tool-id shapeit5" }
    ));
    assert!(
        phasing.get("goal_id").is_none(),
        "VCF micro report must not leak backlog goal ids into repo artifacts"
    );
}
