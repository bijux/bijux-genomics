#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_stage_tool_alias_check_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-stage-tool-alias-check"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "benchmarks/readiness/all-domains/stage-tool-alias-check.json"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read stage-tool-alias-check json");
    let parsed =
        serde_json::from_str::<serde_json::Value>(&payload).expect("parse stage-tool-alias-check");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_tool_alias_check.v1")
    );
    assert_eq!(
        parsed.get("migration_validation_stage_alias_count").and_then(serde_json::Value::as_u64),
        Some(7)
    );
    assert_eq!(parsed.get("tool_alias_cluster_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(parsed.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(parsed.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let migration_aliases = parsed
        .get("migration_validation_stage_aliases")
        .and_then(serde_json::Value::as_array)
        .expect("migration aliases");
    assert_eq!(migration_aliases.len(), 7);
    assert!(migration_aliases.iter().any(|row| {
        row.get("alias_stage_id").and_then(serde_json::Value::as_str) == Some("validate_pre")
            && row.get("canonical_stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.validate_reads")
    }));
    assert!(migration_aliases.iter().any(|row| {
        row.get("alias_stage_id").and_then(serde_json::Value::as_str) == Some("qc_post")
            && row.get("canonical_stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.report_qc")
    }));

    let tool_alias_clusters = parsed
        .get("tool_alias_clusters")
        .and_then(serde_json::Value::as_array)
        .expect("tool alias clusters");
    assert!(tool_alias_clusters.is_empty());

    let violations =
        parsed.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty());
}
