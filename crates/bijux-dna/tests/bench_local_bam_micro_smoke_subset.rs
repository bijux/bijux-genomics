#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("micro-benchmark-mutators").expect("repo lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    #[cfg(feature = "bam_downstream")]
    let output = {
        let mut command = Command::new("cargo");
        command.args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"]);
        command
            .current_dir(&repo_root)
            .env("HOME", home.path())
            .env("BIJUX_SKIP_QA", "1")
            .env("BIJUX_ALLOW_SILVER", "1")
            .env("BIJUX_SKIP_IMAGE_CHECK", "1")
            .args(args)
            .output()
            .expect("run cli")
    };

    #[cfg(not(feature = "bam_downstream"))]
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
fn bench_local_bam_micro_smoke_subset_reports_one_governed_row_per_family() {
    let payload = run_cli_json(&["bench", "local", "run-bam-micro-smoke-subset", "--json"]);
    let expected_local_smoke_count = if cfg!(feature = "bam_downstream") { 10 } else { 9 };
    let expected_container_needed_count = 12 - expected_local_smoke_count;

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_bam_micro_smoke_subset.v2")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/bam/MICRO_BAM_SUMMARY.json")
    );
    assert_eq!(payload.get("family_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(
        payload.get("local_smoke_count").and_then(serde_json::Value::as_u64),
        Some(expected_local_smoke_count)
    );
    assert_eq!(
        payload.get("container_needed_count").and_then(serde_json::Value::as_u64),
        Some(expected_container_needed_count)
    );
    assert_eq!(payload.get("unavailable_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 12);

    let family_ids = rows
        .iter()
        .filter_map(|row| row.get("family_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        family_ids,
        BTreeSet::from([
            "bam.align",
            "bam.complexity",
            "bam.contamination_sex_haplogroups",
            "bam.coverage",
            "bam.damage_authenticity",
            "bam.duplicate_handling",
            "bam.filtering",
            "bam.insert_size_gc_bias",
            "bam.kinship",
            "bam.overlap_endogenous_content",
            "bam.recalibration_genotyping",
            "bam.validation_core_qc",
        ])
    );

    let align = rows
        .iter()
        .find(|row| row.get("family_id").and_then(serde_json::Value::as_str) == Some("bam.align"))
        .expect("bam.align family row");
    assert_eq!(
        align.get("execution_status").and_then(serde_json::Value::as_str),
        Some("container_needed")
    );
    assert_eq!(
        align.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("bam.align")
    );
    assert_eq!(
        align.get("representative_tool_id").and_then(serde_json::Value::as_str),
        Some("bwa")
    );
    assert!(align
        .get("smoke_command")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|command| command.starts_with("bijux-dna env smoke ")));

    let validation = rows
        .iter()
        .find(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str)
                == Some("bam.validation_core_qc")
        })
        .expect("bam.validation_core_qc family row");
    assert_eq!(
        validation.get("execution_status").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(
        validation.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("bam.validate")
    );
    assert_eq!(
        validation.get("representative_tool_id").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(validation.get("evidence_format").and_then(serde_json::Value::as_str), Some("json"));
    assert_eq!(
        validation.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.validate.local_smoke.report.v1")
    );

    let kinship = rows
        .iter()
        .find(|row| row.get("family_id").and_then(serde_json::Value::as_str) == Some("bam.kinship"))
        .expect("bam.kinship family row");
    let expected_kinship_status =
        if cfg!(feature = "bam_downstream") { "local_smoke" } else { "container_needed" };
    assert_eq!(
        kinship.get("execution_status").and_then(serde_json::Value::as_str),
        Some(expected_kinship_status)
    );
    assert!(
        kinship.get("goal_id").is_none(),
        "BAM micro report must not leak backlog goal ids into repo artifacts"
    );
}
