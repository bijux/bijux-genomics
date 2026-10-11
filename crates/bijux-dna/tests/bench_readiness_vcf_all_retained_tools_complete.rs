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
        .args(["bench", "readiness", "render-vcf-all-retained-tools-complete", "--json"])
        .output()
        .expect("run cli");

    assert!(!output.status.success(), "unadmitted licensed evidence must keep the gate open");
    serde_json::from_slice(
        &std::fs::read(
            repo_root.join(
                "artifacts/benchmark-smoke/readiness/vcf/VCF_ALL_RETAINED_TOOLS_COMPLETE.json",
            ),
        )
        .expect("read report"),
    )
    .expect("parse report as json")
}

#[test]
fn bench_readiness_vcf_all_retained_tools_complete_reports_licensed_evidence_blockers() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_all_retained_tools_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/vcf/VCF_ALL_RETAINED_TOOLS_COMPLETE.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(24));
    let passed_goal_count =
        payload.get("passed_goal_count").and_then(serde_json::Value::as_u64).expect("passed goals");
    let failed_goal_count =
        payload.get("failed_goal_count").and_then(serde_json::Value::as_u64).expect("failed goals");
    assert_eq!(passed_goal_count + failed_goal_count, 24);
    assert!(failed_goal_count >= 3);
    let failing_goal_ids = payload
        .get("failing_goal_ids")
        .and_then(serde_json::Value::as_array)
        .expect("failing goal ids")
        .iter()
        .filter_map(serde_json::Value::as_u64)
        .collect::<std::collections::BTreeSet<_>>();
    let licensed_evidence_blockers = std::collections::BTreeSet::from([339, 340, 349]);
    let allowed_host_tool_blockers =
        std::collections::BTreeSet::from([337, 338, 341, 342, 343, 346]);
    assert!(licensed_evidence_blockers.is_subset(&failing_goal_ids));
    assert!(failing_goal_ids
        .difference(&licensed_evidence_blockers)
        .all(|goal_id| allowed_host_tool_blockers.contains(goal_id)));
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(39));
    assert_eq!(payload.get("retained_stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("retained_tool_count").and_then(serde_json::Value::as_u64), Some(16));
    let active_row_count = payload
        .get("active_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("active row count");
    let removed_row_count = payload
        .get("removed_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("removed row count");
    assert_eq!(active_row_count + removed_row_count, 39);
    assert_eq!(payload.get("active_stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert!(payload
        .get("active_tool_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count > 0));
    for row_count_key in [
        "expected_result_row_count",
        "rendered_command_row_count",
        "parser_fixture_row_count",
        "report_map_row_count",
    ] {
        assert_eq!(
            payload.get(row_count_key).and_then(serde_json::Value::as_u64),
            Some(active_row_count)
        );
    }
    assert_eq!(payload.get("local_smoke_row_count").and_then(serde_json::Value::as_u64), Some(39));
    assert_eq!(
        payload.get("local_smoke_host_stage_row_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload.get("local_smoke_container_row_count").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(
        payload.get("local_smoke_external_artifact_row_count").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(payload.get("ok"), Some(&serde_json::Value::Bool(false)));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert_eq!(checks.len(), 24);
    assert_eq!(
        checks
            .iter()
            .filter(|check| check.get("ok") == Some(&serde_json::Value::Bool(false)))
            .count(),
        usize::try_from(failed_goal_count).expect("failed goal count fits usize")
    );
    for goal_id in [339, 340, 349] {
        assert!(checks.iter().any(|check| {
            check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(goal_id)
                && check.get("ok") == Some(&serde_json::Value::Bool(false))
        }));
    }

    let goal_347 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(347))
        .expect("goal 347 check");
    assert!(
        goal_347
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("every supported backend")),
        "goal 347 detail must keep the retained phasing family explicit"
    );

    let goal_356 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(356))
        .expect("goal 356 check");
    assert!(
        goal_356
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("external-artifact admission")),
        "goal 356 detail must keep the external-artifact admission split explicit"
    );
}
