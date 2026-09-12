use anyhow::{Context, Result};
use bijux_dna_bench_fastq::proofs::qc_fixture::{materialize_qc_fixture, LocalReportQcConfig};
use std::{
    fs,
    path::{Path, PathBuf},
};
const LOCAL_REPORT_QC_CONFIG_PATH: &str = "configs/bench/local/fastq-report-qc.toml";
const DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/fastq.report_qc";

pub(super) fn materialize_local_report_qc_smoke_report(repo_root: &Path) -> Result<PathBuf> {
    let config = load_local_report_qc_config(repo_root)?;
    let output_root = repo_root.join(
        config
            .output_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR)),
    );
    crate::commands::benchmark::path_resolution::ensure_path_stays_outside_benchmark_readiness_root(repo_root,&output_root,"local fastq.report_qc smoke output")?;
    materialize_qc_fixture(
        repo_root,
        &repo_root.join(LOCAL_REPORT_QC_CONFIG_PATH),
        config,
        &output_root,
    )
}

fn load_local_report_qc_config(repo_root: &Path) -> Result<LocalReportQcConfig> {
    let config_path = repo_root.join(LOCAL_REPORT_QC_CONFIG_PATH);
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}
