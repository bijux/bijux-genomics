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
        .args(["bench", "readiness", "render-bam-recalibration-genotyping-ready", "--json"])
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
fn bench_readiness_bam_recalibration_genotyping_ready_reports_complete_governed_rows() {
    let payload = run_cli_json();

    #[cfg(feature = "bam_downstream")]
    let expected_row_count = 2;
    #[cfg(not(feature = "bam_downstream"))]
    let expected_row_count = 1;

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_recalibration_genotyping_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/recalibration-genotyping-ready.json")
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
        Some(expected_row_count)
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

    let recalibration_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.recalibration")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("gatk")
        })
        .expect("bam.recalibration gatk row");
    assert_eq!(
        recalibration_row
            .get("expected_normalized_metrics_output_id")
            .and_then(serde_json::Value::as_str),
        Some("recal_report")
    );
    assert_eq!(
        recalibration_row.get("expected_local_proof_kind").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(
        recalibration_row.get("local_proof_tool_id").and_then(serde_json::Value::as_str),
        Some("gatk")
    );
    assert!(recalibration_row
        .get("required_local_proof_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("known_sites"))
                && fields.iter().any(|field| field.as_str() == Some("observed_mean_coverage"))
        }));
    assert!(recalibration_row
        .get("local_proof_declared_output_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|output| output.as_str() == Some("recalibration_summary"))
        }));

    #[cfg(feature = "bam_downstream")]
    {
        let genotyping_row = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
            })
            .expect("bam.genotyping angsd row");
        assert_eq!(
            genotyping_row
                .get("expected_normalized_metrics_output_id")
                .and_then(serde_json::Value::as_str),
            Some("genotyping_report")
        );
        assert_eq!(
            genotyping_row.get("expected_local_proof_kind").and_then(serde_json::Value::as_str),
            Some("local_ready_plan")
        );
        assert_eq!(
            genotyping_row.get("local_proof_tool_id").and_then(serde_json::Value::as_str),
            Some("angsd")
        );
        assert!(genotyping_row
            .get("required_local_proof_fields")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|fields| {
                fields.iter().any(|field| field.as_str() == Some("min_call_rate"))
                    && fields.iter().any(|field| field.as_str() == Some("producer_contract"))
            }));
        assert!(genotyping_row
            .get("local_proof_declared_output_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| {
                outputs.iter().any(|output| output.as_str() == Some("genotyping_bcf"))
                    && outputs.iter().any(|output| output.as_str() == Some("genotyping_gl"))
            }));
        assert!(genotyping_row
            .get("local_proof_command_shell")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|shell| {
                shell.contains("human_like_genotyping_candidate_sites.vcf")
                    && shell.contains("genotyping.vcf.gz")
            }));
    }
}
