#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_damage_complete_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-bam-damage-complete"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/bam/stages/bam.damage.complete.json");

    let payload = fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM damage completion JSON");
    let report: serde_json::Value =
        serde_json::from_str(&payload).expect("parse BAM damage completion JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_damage_complete.v1")
    );
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(report.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(report.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));

    assert!(report.get("required_output_ids").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| {
            outputs.iter().any(|output| output.as_str() == Some("damage_report"))
                && outputs.iter().any(|output| output.as_str() == Some("terminal_position_metrics"))
                && outputs.iter().any(|output| output.as_str() == Some("parser_output"))
                && outputs.iter().any(|output| output.as_str() == Some("stage_metrics"))
        }
    ));
    assert!(report
        .get("required_parser_output_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("schema_version"))
                && fields.iter().any(|field| field.as_str() == Some("parsed_tools"))
        }));
    assert!(report
        .get("required_normalized_metric_names")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("terminal_c_to_t_5p_delta"))
                && fields.iter().any(|field| field.as_str() == Some("damage_signal"))
        }));
}
