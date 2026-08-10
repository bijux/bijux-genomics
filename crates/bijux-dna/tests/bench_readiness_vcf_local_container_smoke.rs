#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
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
fn bench_readiness_vcf_local_container_smoke_reports_retained_wrapper_paths() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-local-container-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_local_container_smoke.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/vcf/vcf-local-container-smoke.tsv")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let row_count = u64::try_from(rows.len()).expect("row count fits u64");
    let stage_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("stage_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("stage count fits u64");
    let tool_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("tool_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("tool count fits u64");
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(stage_count));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(tool_count));
    let host_row_count = rows
        .iter()
        .filter(|row| {
            row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
        })
        .count();
    let host_row_count = u64::try_from(host_row_count).expect("host row count fits u64");
    assert_eq!(
        payload.get("host_stage_smoke_row_count").and_then(serde_json::Value::as_u64),
        Some(host_row_count)
    );
    assert_eq!(
        payload.get("container_smoke_row_count").and_then(serde_json::Value::as_u64),
        Some(
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.get("smoke_path_kind").and_then(serde_json::Value::as_str),
                        Some("docker_container_smoke" | "apptainer_container_smoke")
                    )
                })
                .count() as u64
        )
    );
    let external_rows = rows
        .iter()
        .filter(|row| {
            row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("external_artifact_admission")
        })
        .count() as u64;
    assert_eq!(
        payload.get("external_artifact_row_count").and_then(serde_json::Value::as_u64),
        Some(external_rows)
    );
    assert_eq!(external_rows, 4);

    let runtime_counts = payload
        .get("runtime_counts")
        .and_then(serde_json::Value::as_object)
        .expect("runtime counts");
    let runtime_row_count =
        runtime_counts.values().map(|count| count.as_u64().expect("runtime count")).sum::<u64>();
    assert_eq!(runtime_row_count, row_count);
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("host")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-vcf-roh-smoke --tool-id plink2")
    }));
    for (stage_id, tool_id) in [
        ("vcf.call_gl", "angsd"),
        ("vcf.call_pseudohaploid", "angsd"),
        ("vcf.imputation_metrics", "impute5"),
        ("vcf.impute", "impute5"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                    == Some("external_artifact_admission")
                && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("external")
        }));
    }

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("registered_binary").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("host")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-vcf-call-smoke --tool-id bcftools")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("germline")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-vcf-ibd-smoke --tool-id germline")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation_metrics")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("registered_binary").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("host")
            && row.get("smoke_tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-vcf-imputation-metrics-smoke --tool-id beagle")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-vcf-impute-smoke --tool-id beagle")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("host_stage_smoke")
            && row.get("smoke_runtime").and_then(serde_json::Value::as_str) == Some("host")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna bench local run-vcf-postprocess-smoke --tool-id bcftools")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit")
            && row.get("smoke_path_kind").and_then(serde_json::Value::as_str)
                == Some("apptainer_container_smoke")
            && row.get("smoke_command").and_then(serde_json::Value::as_str)
                == Some("bijux-dna env smoke apptainer shapeit")
            && row.get("smoke_minimal_cmd").and_then(serde_json::Value::as_str) == Some("")
    }));
}
