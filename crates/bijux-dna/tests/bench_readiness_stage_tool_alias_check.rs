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
fn bench_readiness_stage_tool_alias_check_reports_migration_only_aliases() {
    let payload = run_cli_json(&["bench", "readiness", "render-stage-tool-alias-check", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_tool_alias_check.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/stage-tool-alias-check.json")
    );
    assert_eq!(
        payload.get("migration_validation_stage_alias_count").and_then(serde_json::Value::as_u64),
        Some(7)
    );
    assert_eq!(
        payload.get("tool_alias_cluster_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("candidate_row_count").and_then(serde_json::Value::as_u64), Some(145));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(125));
    assert_eq!(
        payload.get("expected_result_row_count").and_then(serde_json::Value::as_u64),
        Some(125)
    );
    assert_eq!(
        payload.get("rendered_command_row_count").and_then(serde_json::Value::as_u64),
        Some(125)
    );
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let surface_violation_counts = payload
        .get("surface_violation_counts")
        .and_then(serde_json::Value::as_object)
        .expect("surface violation counts");
    assert!(
        surface_violation_counts.is_empty(),
        "governed candidate, active, expected-result, and rendered-command surfaces must stay alias-free"
    );

    let alias_kind_violation_counts = payload
        .get("alias_kind_violation_counts")
        .and_then(serde_json::Value::as_object)
        .expect("alias kind violation counts");
    assert!(
        alias_kind_violation_counts.is_empty(),
        "stage and tool alias violations must stay empty on governed active surfaces"
    );

    let migration_aliases = payload
        .get("migration_validation_stage_aliases")
        .and_then(serde_json::Value::as_array)
        .expect("migration aliases");
    assert_eq!(migration_aliases.len(), 7);
    assert!(migration_aliases.iter().all(|row| {
        row.get("accepted_scope").and_then(serde_json::Value::as_str)
            == Some("migration_validation_only")
    }));
    assert!(migration_aliases.iter().any(|row| {
        row.get("alias_stage_id").and_then(serde_json::Value::as_str) == Some("report_qc")
            && row.get("canonical_stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.report_qc")
    }));
    assert!(migration_aliases.iter().any(|row| {
        row.get("alias_stage_id").and_then(serde_json::Value::as_str) == Some("fastq.qc_post")
            && row.get("canonical_stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.report_qc")
    }));

    let tool_alias_clusters = payload
        .get("tool_alias_clusters")
        .and_then(serde_json::Value::as_array)
        .expect("tool alias clusters");
    assert!(
        tool_alias_clusters.is_empty(),
        "governed candidate benchmark tools should not collapse into legacy separator aliases"
    );

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(
        violations.is_empty(),
        "migration-only aliases must stay out of candidate, active, expected-result, and rendered-command surfaces"
    );
}
