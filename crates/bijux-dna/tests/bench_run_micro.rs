#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_run_micro_includes_fastq_family_component() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("micro-benchmark-mutators").expect("repo lock");
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
        .args(["bench", "run-micro", "--json"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_micro_benchmark_run.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/MICRO_BENCHMARK_RUN.json")
    );
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let components = payload
        .get("component_reports")
        .and_then(serde_json::Value::as_array)
        .expect("component reports");
    let component_ids = components
        .iter()
        .filter_map(|component| component.get("component_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        component_ids,
        BTreeSet::from([
            "amplicon_micro_pipeline",
            "adna_micro_pipeline",
            "bam_micro_smoke_subset",
            "core_germline_micro_pipeline",
            "edna_micro_pipeline",
            "fastq_micro_smoke_subset",
            "real_smoke_core_subset",
            "vcf_micro_smoke_subset",
        ])
    );
    assert!(components.iter().any(|component| {
        component.get("component_id").and_then(serde_json::Value::as_str)
            == Some("fastq_micro_smoke_subset")
            && component.get("report_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json")
    }));
    assert!(components.iter().any(|component| {
        component.get("component_id").and_then(serde_json::Value::as_str)
            == Some("bam_micro_smoke_subset")
            && component.get("report_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/micro/bam/MICRO_BAM_SUMMARY.json")
    }));
    assert!(components.iter().any(|component| {
        component.get("component_id").and_then(serde_json::Value::as_str)
            == Some("vcf_micro_smoke_subset")
            && component.get("report_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json")
    }));
    assert!(components.iter().any(|component| {
        component.get("component_id").and_then(serde_json::Value::as_str)
            == Some("amplicon_micro_pipeline")
            && component.get("report_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json")
    }));
    assert!(components.iter().any(|component| {
        component.get("component_id").and_then(serde_json::Value::as_str)
            == Some("adna_micro_pipeline")
            && component.get("report_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json")
    }));
    assert!(components.iter().any(|component| {
        component.get("component_id").and_then(serde_json::Value::as_str)
            == Some("edna_micro_pipeline")
            && component.get("report_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/micro/pipelines/edna/MICRO_EDNA_SUMMARY.json")
    }));
    assert!(components.iter().any(|component| {
        component.get("component_id").and_then(serde_json::Value::as_str)
            == Some("core_germline_micro_pipeline")
            && component.get("report_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json")
    }));
}
