#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::process::Command;

use bijux_dna_domain_vcf::VCF_STAGE_ORDER_DOWNSTREAM;

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
fn bench_local_vcf_smoke_root_reports_governed_stage_tool_paths() {
    let payload = run_cli_json(&["bench", "local", "render-vcf-smoke-root", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_smoke_root.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf/SMOKE_ROOT.json")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local render-vcf-smoke-root")
    );
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("tool_pair_count").and_then(serde_json::Value::as_u64), Some(20));
    assert!(payload
        .get("run_id")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.starts_with("vcf-local-smoke-")));
    assert!(payload
        .get("repo_revision")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.len() == 40));
    assert!(payload
        .get("created_at")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty()));
    assert!(payload.get("worktree_dirty").and_then(serde_json::Value::as_bool).is_some());

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);

    let observed_stage_ids = rows
        .iter()
        .map(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str).expect("stage_id").to_string()
        })
        .collect::<Vec<_>>();
    let expected_order = VCF_STAGE_ORDER_DOWNSTREAM
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(observed_stage_ids, expected_order);
    assert_eq!(
        observed_stage_ids.iter().cloned().collect::<BTreeSet<_>>().len(),
        VCF_STAGE_ORDER_DOWNSTREAM.len()
    );

    let prepare_reference_panel = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
        })
        .expect("prepare reference panel row");
    assert_eq!(
        prepare_reference_panel.get("tool_id").and_then(serde_json::Value::as_str),
        Some("bcftools")
    );
    assert_eq!(
        prepare_reference_panel.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("vcf_reference_panel")
    );
    assert_eq!(
        prepare_reference_panel.get("pair_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf/vcf.prepare_reference_panel/bcftools")
    );
    assert_eq!(
        prepare_reference_panel.get("artifacts_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf/vcf.prepare_reference_panel/bcftools/artifacts")
    );
    assert_eq!(
        prepare_reference_panel.get("result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf/vcf.prepare_reference_panel/bcftools/stage-result.json")
    );

    let phasing = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing"))
        .expect("phasing row");
    assert_eq!(phasing.get("tool_id").and_then(serde_json::Value::as_str), Some("shapeit5"));
    assert_eq!(
        phasing.get("local_smoke_mode").and_then(serde_json::Value::as_str),
        Some("vcf_cohort_with_panel")
    );
    assert_eq!(
        phasing.get("expected_outputs").and_then(serde_json::Value::as_array).map(|values| {
            values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
        }),
        Some(vec!["phased_vcf"])
    );
}
