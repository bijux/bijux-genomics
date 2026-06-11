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
fn bench_readiness_vcf_active_stage_tool_matrix_reports_every_retained_binding() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-active-stage-tool-matrix", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_active_stage_tool_matrix.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/vcf-active-stage-tool-matrix.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(44));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(28));

    let scope_state_counts = payload
        .get("scope_state_counts")
        .and_then(serde_json::Value::as_object)
        .expect("scope state counts");
    assert_eq!(scope_state_counts.get("active").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(
        scope_state_counts.get("removed_from_scope").and_then(serde_json::Value::as_u64),
        Some(28)
    );
    assert!(
        scope_state_counts.get("complete").is_none(),
        "current governed VCF retained surface should not claim hidden complete rows"
    );

    let scope_detail_counts = payload
        .get("scope_detail_counts")
        .and_then(serde_json::Value::as_object)
        .expect("scope detail counts");
    assert_eq!(scope_detail_counts.get("active").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(
        scope_detail_counts.get("benchmark_not_ready").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(
        scope_detail_counts.get("lifecycle_not_active").and_then(serde_json::Value::as_u64),
        Some(13)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 44);

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call_gl")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("planned")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str)
                == Some("removed_from_scope")
            && row.get("scope_detail").and_then(serde_json::Value::as_str)
                == Some("benchmark_not_ready")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("vcf.prepare_reference_panel")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_reference_panel")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation_metrics")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle-imputation")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("experimental")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str) == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str)
                == Some("removed_from_scope")
            && row.get("scope_detail").and_then(serde_json::Value::as_str)
                == Some("benchmark_not_ready")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eagle")
            && row.get("tool_status").and_then(serde_json::Value::as_str)
                == Some("experimental,planned")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str)
                == Some("removed_from_scope")
            && row.get("scope_detail").and_then(serde_json::Value::as_str)
                == Some("benchmark_not_ready")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json")
    }));
}
