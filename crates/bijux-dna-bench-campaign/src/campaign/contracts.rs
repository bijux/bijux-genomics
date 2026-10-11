#![allow(clippy::expect_used)]
use super::admission::resolve_tools;
use super::paths::{benchmark_runtime_env, workspace_cache_root_for_output};
use super::resume::sample_report_is_resume_ready;
use std::{fs, path::PathBuf};
#[test]
fn tool_resolution_rejects_contract_drift() {
    let error =
        resolve_tools(&["fastqc".to_string(), "seqtk".to_string()], &["fastqc".to_string()])
            .expect_err("tool drift should fail");
    assert!(error.to_string().contains("tool roster drift"));
}

#[test]
fn resume_ready_requires_successful_gate_and_records() {
    let temp = tempfile::tempdir().expect("tempdir");
    let report = temp.path().join("report.json");
    fs::write(
        &report,
        serde_json::json!({
            "failures": [],
            "gate": {"passes": true},
            "records": [{"tool": "fastp"}],
        })
        .to_string(),
    )
    .expect("write report");
    assert!(sample_report_is_resume_ready(&report));

    fs::write(
        &report,
        serde_json::json!({
            "failures": [{"kind": "tool_exit"}],
            "gate": {"passes": false},
            "records": [],
        })
        .to_string(),
    )
    .expect("write failed report");
    assert!(!sample_report_is_resume_ready(&report));
}

#[test]
fn benchmark_runtime_env_follows_cache_root_ancestor() {
    let out_root = PathBuf::from(
        "/bench/workspace/.cache/results/benchmark_corpus/fastq.trim_reads/cluster-apptainer",
    );
    let env = benchmark_runtime_env(&out_root);
    assert_eq!(env.get("BIJUX_CACHE_ROOT"), Some(&"/bench/workspace/.cache".to_string()));
    assert_eq!(env.get("XDG_CACHE_HOME"), Some(&"/bench/workspace/.cache".to_string()));
    assert!(!env.contains_key("BIJUX_HPC_ROOT"));
    assert_eq!(
        workspace_cache_root_for_output(&out_root),
        Some(PathBuf::from("/bench/workspace/.cache"))
    );
}
