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
        .args(["bench", "readiness", "render-bam-active-row-consistency", "--json"])
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
fn bench_readiness_bam_active_row_consistency_reports_governed_surface_alignment() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_active_row_consistency.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/BAM_ACTIVE_ROW_CONSISTENCY.json")
    );
    assert_eq!(
        payload.get("active_matrix_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    );
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("passed_surface_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("failed_surface_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let active_row_count =
        payload.get("active_row_count").and_then(serde_json::Value::as_u64).expect("row count");
    let active_stage_count =
        payload.get("active_stage_count").and_then(serde_json::Value::as_u64).expect("stage count");
    let active_tool_count =
        payload.get("active_tool_count").and_then(serde_json::Value::as_u64).expect("tool count");

    assert!(
        active_row_count >= 47,
        "active BAM rows unexpectedly shrank below the retained matrix"
    );
    assert!(
        active_stage_count >= 23,
        "active BAM stage count unexpectedly shrank below the retained stage set"
    );
    assert!(
        active_tool_count >= 23,
        "active BAM tool count unexpectedly shrank below the retained tool set"
    );

    let surfaces = payload.get("surfaces").and_then(serde_json::Value::as_array).expect("surfaces");
    assert_eq!(surfaces.len(), 6);

    for expected_surface_id in [
        "bam_rendered_commands",
        "bam_output_declarations",
        "bam_expected_results",
        "bam_parser_fixtures",
        "bam_local_jobs",
        "bam_report_map",
    ] {
        let surface = surfaces
            .iter()
            .find(|surface| {
                surface.get("surface_id").and_then(serde_json::Value::as_str)
                    == Some(expected_surface_id)
            })
            .unwrap_or_else(|| {
                panic!("missing governed BAM consistency surface `{expected_surface_id}`")
            });

        assert_eq!(surface.get("ok").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            surface.get("row_count").and_then(serde_json::Value::as_u64),
            Some(active_row_count)
        );
        assert_eq!(
            surface.get("binding_count").and_then(serde_json::Value::as_u64),
            Some(active_row_count)
        );
        assert_eq!(
            surface.get("stage_count").and_then(serde_json::Value::as_u64),
            Some(active_stage_count)
        );
        assert_eq!(
            surface.get("tool_count").and_then(serde_json::Value::as_u64),
            Some(active_tool_count)
        );
        assert_eq!(
            surface.get("missing_bindings").and_then(serde_json::Value::as_array).map(Vec::len),
            Some(0)
        );
        assert_eq!(
            surface.get("extra_bindings").and_then(serde_json::Value::as_array).map(Vec::len),
            Some(0)
        );
    }

    assert_eq!(
        payload.get("failed_surfaces").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
}
