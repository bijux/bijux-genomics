#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use crate::support;

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

fn retained_registry_bindings(repo_root: &Path) -> BTreeSet<(String, String)> {
    let mut bindings = BTreeSet::new();
    for relative_path in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let raw = std::fs::read_to_string(repo_root.join(relative_path)).expect("read registry");
        let parsed: toml::Value = toml::from_str(&raw).expect("parse registry");
        for tool in parsed["tools"].as_array().expect("registry tools") {
            if tool["status"].as_str() == Some("unsupported") {
                continue;
            }
            let tool_id = tool["id"].as_str().expect("tool id");
            for stage_id in tool["stage_ids"].as_array().expect("tool stage ids") {
                bindings.insert((
                    stage_id.as_str().expect("stage id").to_string(),
                    tool_id.to_string(),
                ));
            }
        }
    }
    bindings
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
        Some("artifacts/benchmark-smoke/readiness/vcf/vcf-active-stage-tool-matrix.tsv")
    );
    let row_count = payload["row_count"].as_u64().expect("row count");
    let active_row_count = payload["active_row_count"].as_u64().expect("active row count");
    let complete_row_count = payload["complete_row_count"].as_u64().expect("complete row count");
    let removed_row_count = payload["removed_row_count"].as_u64().expect("removed row count");
    assert!(payload["stage_count"].as_u64().is_some_and(|count| count > 0));
    assert!(payload["tool_count"].as_u64().is_some_and(|count| count > 0));
    assert_eq!(active_row_count + complete_row_count + removed_row_count, row_count);

    let scope_state_counts = payload
        .get("scope_state_counts")
        .and_then(serde_json::Value::as_object)
        .expect("scope state counts");
    assert_eq!(
        scope_state_counts.get("active").and_then(serde_json::Value::as_u64),
        Some(active_row_count)
    );
    assert_eq!(
        scope_state_counts
            .get("removed_from_scope")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default(),
        removed_row_count
    );
    assert!(
        scope_state_counts.get("complete").is_none(),
        "current governed VCF retained surface should not claim hidden complete rows"
    );

    let scope_detail_counts = payload
        .get("scope_detail_counts")
        .and_then(serde_json::Value::as_object)
        .expect("scope detail counts");
    assert_eq!(
        scope_detail_counts.get("active").and_then(serde_json::Value::as_u64),
        Some(active_row_count)
    );
    let removed_detail_count = scope_detail_counts
        .get("benchmark_not_ready")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default()
        + scope_detail_counts
            .get("lifecycle_not_active")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
    assert_eq!(removed_detail_count, removed_row_count);

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len() as u64, row_count);
    assert!(rows.iter().all(|row| {
        matches!(
            row.get("tool_status").and_then(serde_json::Value::as_str),
            Some("production" | "supported")
        )
    }));
    let actual_bindings = rows
        .iter()
        .map(|row| {
            (
                row["stage_id"].as_str().expect("row stage id").to_string(),
                row["tool_id"].as_str().expect("row tool id").to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_bindings.len(), rows.len());
    assert_eq!(
        actual_bindings,
        retained_registry_bindings(&support::repo_root().expect("repo root"))
    );

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eigensoft")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
    }));
    assert!(rows.iter().all(|row| {
        row.get("tool_status").and_then(serde_json::Value::as_str) != Some("unsupported")
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
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
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
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
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
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
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
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eagle")
            && row.get("tool_status").and_then(serde_json::Value::as_str) == Some("production")
            && row.get("stage_support_status").and_then(serde_json::Value::as_str)
                == Some("supported")
            && row.get("scope_state").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_detail").and_then(serde_json::Value::as_str) == Some("active")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv",
                )
    }));
}
