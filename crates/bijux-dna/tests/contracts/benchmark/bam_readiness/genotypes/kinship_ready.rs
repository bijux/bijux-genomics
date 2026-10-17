#![allow(clippy::expect_used, clippy::too_many_lines)]

use crate::support;

fn run_cli_json() -> serde_json::Value {
    let sandbox =
        support::RepoSandbox::new_bam_genotype("bam-kinship-ready-").expect("repo sandbox");
    let runtime_root = sandbox.path().join("artifacts/test-runtime");
    std::fs::create_dir_all(&runtime_root).expect("private runtime root");
    let home = tempfile::Builder::new()
        .prefix("bam-genotype-runtime-")
        .tempdir_in(&runtime_root)
        .expect("private runtime directory");

    let output = sandbox
        .bijux_dna_command()
        .envs([
            ("HOME", home.path()),
            ("TMPDIR", home.path()),
            ("TMP", home.path()),
            ("TEMP", home.path()),
        ])
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-bam-kinship-ready", "--json"])
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
fn bench_readiness_bam_kinship_ready_reports_complete_governed_rows() {
    let payload = run_cli_json();

    #[cfg(feature = "bam_downstream")]
    let expected_row_count = 1;
    #[cfg(not(feature = "bam_downstream"))]
    let expected_row_count = 0;

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_kinship_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/bam/kinship-ready.json")
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
        Some(u64::from(expected_row_count > 0))
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

    #[cfg(feature = "bam_downstream")]
    {
        let king_row = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("king")
            })
            .expect("bam.kinship king row");
        assert_eq!(
            king_row
                .get("expected_normalized_metrics_output_id")
                .and_then(serde_json::Value::as_str),
            Some("kinship_report")
        );
        assert_eq!(
            king_row.get("expected_local_proof_kind").and_then(serde_json::Value::as_str),
            Some("local_smoke")
        );
        assert_eq!(
            king_row.get("local_proof_tool_id").and_then(serde_json::Value::as_str),
            Some("king")
        );
        assert_eq!(
            king_row.get("local_proof_sample_id").and_then(serde_json::Value::as_str),
            Some("sample-set")
        );
        assert!(king_row
            .get("local_proof_declared_output_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| {
                outputs.iter().any(|output| output.as_str() == Some("kinship_report"))
                    && outputs.iter().any(|output| output.as_str() == Some("summary"))
                    && outputs.iter().any(|output| output.as_str() == Some("stage_metrics"))
            }));
        assert!(king_row
            .get("required_local_proof_fields")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|fields| {
                fields.iter().any(|field| field.as_str() == Some("overlap_snps"))
                    && fields.iter().any(|field| field.as_str() == Some("kinship_coefficient"))
                    && fields.iter().any(|field| field.as_str() == Some("status"))
            }));
        assert!(king_row
            .get("local_proof_artifact_paths")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|paths| {
                paths.iter().any(|path| {
                    path.as_str().is_some_and(|value| value.ends_with("kinship.segments.tsv"))
                })
            }));
        assert!(rows
            .iter()
            .all(|row| row.get("tool_id").and_then(serde_json::Value::as_str) != Some("angsd")));
    }
}
