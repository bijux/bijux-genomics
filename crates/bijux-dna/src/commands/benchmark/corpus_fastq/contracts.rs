#![allow(clippy::expect_used)]
use super::{default_stage_out_root, resolve_benchmark_platform};
use crate::commands::benchmark_workspace::{
    BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout, BenchmarkWorkspaceRemote,
    BenchmarkWorkspaceStageRuns,
};
use std::path::PathBuf;
#[test]
fn default_out_root_uses_workspace_template() {
    let workspace = BenchmarkWorkspaceConfig {
        remote: Some(BenchmarkWorkspaceRemote {
            corpus_root: Some("/bench/remote/corpora/benchmark_corpus".to_string()),
            results_root: Some("/bench/remote/cache/results".to_string()),
            ..BenchmarkWorkspaceRemote::default()
        }),
        layout: Some(BenchmarkWorkspaceLayout {
            stage_runs: Some(BenchmarkWorkspaceStageRuns {
                remote_results_template: Some("{corpus_id}/{stage_id}/cluster".to_string()),
                ..BenchmarkWorkspaceStageRuns::default()
            }),
        }),
        ..BenchmarkWorkspaceConfig::default()
    };
    let out_root = default_stage_out_root(&workspace, "benchmark-corpus", "fastq.validate_reads")
        .expect("root");
    assert_eq!(
        out_root,
        PathBuf::from("/bench/remote/cache/results/benchmark_corpus/fastq.validate_reads/cluster",)
    );
}

#[test]
fn resolve_benchmark_platform_rejects_empty_identifier() {
    let error =
        resolve_benchmark_platform(Some("   ")).expect_err("empty platform contract must fail");
    assert!(error.to_string().contains("benchmark platform must be a non-empty identifier"));
}
