#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_contamination_sex_haplogroups_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-bam-contamination-sex-haplogroups-ready"])
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
        "benchmarks/readiness/bam/contamination-sex-haplogroups-ready.json"
    );

    let payload = fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM contamination/sex/haplogroups readiness JSON");
    let report: serde_json::Value = serde_json::from_str(&payload)
        .expect("parse BAM contamination/sex/haplogroups readiness JSON");

    #[cfg(feature = "bam_downstream")]
    let expected_row_count = 7;
    #[cfg(not(feature = "bam_downstream"))]
    let expected_row_count = 6;

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_contamination_sex_haplogroups_ready.v1")
    );
    assert_eq!(
        report.get("complete_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_row_count)
    );
    assert_eq!(report.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));

    let expected_tool_ids = report
        .get("expected_tool_ids_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("expected tool ids by stage");
    assert!(expected_tool_ids
        .get("bam.contamination")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|tools| {
            tools.iter().any(|tool| tool.as_str() == Some("contammix"))
                && tools.iter().any(|tool| tool.as_str() == Some("schmutzi"))
                && tools.iter().any(|tool| tool.as_str() == Some("verifybamid2"))
        }));
    assert!(expected_tool_ids.get("bam.sex").and_then(serde_json::Value::as_array).is_some_and(
        |tools| {
            tools.iter().any(|tool| tool.as_str() == Some("angsd"))
                && tools.iter().any(|tool| tool.as_str() == Some("rxy"))
                && tools.iter().any(|tool| tool.as_str() == Some("yleaf"))
        }
    ));

    #[cfg(feature = "bam_downstream")]
    assert!(expected_tool_ids
        .get("bam.haplogroups")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|tools| tools.iter().any(|tool| tool.as_str() == Some("yleaf"))));

    let expected_local_proof_kind = report
        .get("expected_local_proof_kind_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("expected local proof kind by stage");
    assert_eq!(
        expected_local_proof_kind.get("bam.contamination").and_then(serde_json::Value::as_str),
        Some("local_ready_plan")
    );
    assert_eq!(
        expected_local_proof_kind.get("bam.sex").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );

    let required_output_ids = report
        .get("required_output_ids_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("required output ids by stage");
    assert!(required_output_ids
        .get("bam.contamination")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|output| output.as_str() == Some("contamination_report"))
                && outputs.iter().any(|output| output.as_str() == Some("summary"))
                && outputs.iter().any(|output| output.as_str() == Some("stage_metrics"))
        }));
    assert!(required_output_ids.get("bam.sex").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| {
            outputs.iter().any(|output| output.as_str() == Some("sex_report"))
                && outputs.iter().any(|output| output.as_str() == Some("summary"))
                && outputs.iter().any(|output| output.as_str() == Some("stage_metrics"))
        }
    ));
}
