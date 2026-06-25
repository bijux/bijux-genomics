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
        .args(["bench", "readiness", "render-bam-contamination-sex-haplogroups-ready", "--json"])
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
fn bench_readiness_bam_contamination_sex_haplogroups_ready_reports_complete_governed_rows() {
    let payload = run_cli_json();

    #[cfg(feature = "bam_downstream")]
    let expected_row_count = 7;
    #[cfg(not(feature = "bam_downstream"))]
    let expected_row_count = 6;

    #[cfg(feature = "bam_downstream")]
    let expected_stage_count = 3;
    #[cfg(not(feature = "bam_downstream"))]
    let expected_stage_count = 2;

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_contamination_sex_haplogroups_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/contamination-sex-haplogroups-ready.json")
    );
    assert_eq!(
        payload.get("active_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_row_count)
    );
    assert_eq!(
        payload.get("complete_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_row_count)
    );
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        payload.get("stage_count").and_then(serde_json::Value::as_u64),
        Some(expected_stage_count)
    );
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len() as u64, expected_row_count);
    assert!(rows.iter().all(|row| {
        row.get("active_scope_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("command_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("output_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("parser_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("expected_result_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("schema_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("local_proof_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("complete")
    }));

    let contamination_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("verifybamid2")
        })
        .expect("bam.contamination verifybamid2 row");
    assert_eq!(
        contamination_row
            .get("expected_normalized_metrics_output_id")
            .and_then(serde_json::Value::as_str),
        Some("contamination_report")
    );
    assert_eq!(
        contamination_row.get("expected_local_proof_kind").and_then(serde_json::Value::as_str),
        Some("local_ready_plan")
    );
    assert_eq!(
        contamination_row.get("local_proof_tool_id").and_then(serde_json::Value::as_str),
        Some("verifybamid2")
    );
    assert!(contamination_row
        .get("required_local_proof_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("reference_panels"))
                && fields.iter().any(|field| field.as_str() == Some("scope"))
        }));
    assert!(contamination_row
        .get("local_proof_declared_output_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs.iter().any(|output| output.as_str() == Some("summary"))));

    let sex_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("rxy")
        })
        .expect("bam.sex rxy row");
    assert_eq!(
        sex_row.get("expected_normalized_metrics_output_id").and_then(serde_json::Value::as_str),
        Some("sex_report")
    );
    assert_eq!(
        sex_row.get("expected_local_proof_kind").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(sex_row.get("local_proof_tool_id").and_then(serde_json::Value::as_str), Some("rxy"));
    assert!(sex_row
        .get("required_local_proof_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("autosomal_coverage"))
                && fields.iter().any(|field| field.as_str() == Some("call"))
        }));
    assert!(sex_row
        .get("local_proof_artifact_paths")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|paths| {
            paths
                .iter()
                .any(|path| path.as_str() == Some("runs/bench/local-smoke/bam.sex/sex.json"))
        }));

    #[cfg(feature = "bam_downstream")]
    {
        let haplogroups_row = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.haplogroups")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("yleaf")
            })
            .expect("bam.haplogroups yleaf row");
        assert_eq!(
            haplogroups_row
                .get("expected_normalized_metrics_output_id")
                .and_then(serde_json::Value::as_str),
            Some("haplogroups")
        );
        assert_eq!(
            haplogroups_row.get("expected_local_proof_kind").and_then(serde_json::Value::as_str),
            Some("local_ready_plan")
        );
        assert_eq!(
            haplogroups_row.get("local_proof_tool_id").and_then(serde_json::Value::as_str),
            Some("yleaf")
        );
        assert!(haplogroups_row
            .get("required_local_proof_fields")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|fields| {
                fields.iter().any(|field| field.as_str() == Some("reference_panel_id"))
                    && fields.iter().any(|field| field.as_str() == Some("coverage_gate"))
            }));
        assert!(haplogroups_row
            .get("local_proof_declared_output_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs
                .iter()
                .any(|output| output.as_str() == Some("summary"))));
    }
}
