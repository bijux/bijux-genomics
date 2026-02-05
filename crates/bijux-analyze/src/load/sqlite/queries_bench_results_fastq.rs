use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use serde_json::Value as JsonValue;

use bijux_core::contract::{BenchResultRecord, BenchResultStatus};
use bijux_domain_fastq::BenchCorpus;

pub trait BenchResultsRepository {
    fn bench_results(
        &self,
        stage: &str,
        tool: &str,
        corpus: &BenchCorpus,
    ) -> Result<Vec<BenchResultRecord>>;
}

#[derive(Debug, Clone)]
pub struct SqliteBenchResultsRepository {
    root_dir: PathBuf,
}

impl SqliteBenchResultsRepository {
    #[must_use]
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }
}

impl BenchResultsRepository for SqliteBenchResultsRepository {
    fn bench_results(
        &self,
        stage: &str,
        tool: &str,
        corpus: &BenchCorpus,
    ) -> Result<Vec<BenchResultRecord>> {
        get_results_from_sqlite(stage, tool, corpus, &self.root_dir)
    }
}

fn table_for_stage(stage: &str) -> Option<&'static str> {
    match stage {
        "fastq.validate_pre" => Some("bench_fastq_validate_v1"),
        "fastq.detect_adapters" => Some("bench_fastq_detect_adapters_v1"),
        "fastq.trim" => Some("bench_fastq_trim_v2"),
        "fastq.filter" => Some("bench_fastq_filter_v2"),
        "fastq.stats_neutral" => Some("bench_fastq_stats_v1"),
        "fastq.merge" => Some("bench_fastq_merge_v1"),
        "fastq.correct" => Some("bench_fastq_correct_v1"),
        "fastq.qc_post" => Some("bench_fastq_qc_post_v1"),
        "fastq.umi" => Some("bench_fastq_umi_v1"),
        "fastq.screen" => Some("bench_fastq_screen_v1"),
        _ => None,
    }
}

fn bench_dir_for_stage(stage: &str) -> Option<&'static str> {
    match stage {
        "fastq.validate_pre" => Some("validate"),
        "fastq.detect_adapters" => Some("detect_adapters"),
        "fastq.trim" => Some("trim"),
        "fastq.filter" => Some("filter"),
        "fastq.stats_neutral" => Some("stats"),
        "fastq.merge" => Some("merge"),
        "fastq.correct" => Some("correct"),
        "fastq.qc_post" => Some("qc_post"),
        "fastq.umi" => Some("umi"),
        "fastq.screen" => Some("screen"),
        _ => None,
    }
}

fn bench_base_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    out.join("artifacts")
        .join("bench")
        .join(stage)
        .join(sample_id)
}

/// Load bench results for a stage/tool across the corpus.
///
/// # Errors
/// Returns an error if the bench database cannot be opened or parsed.
pub fn get_results_from_sqlite(
    stage: &str,
    tool: &str,
    corpus: &BenchCorpus,
    out_dir: &Path,
) -> Result<Vec<BenchResultRecord>> {
    let table = table_for_stage(stage)
        .ok_or_else(|| anyhow!("unsupported stage for bench query: {stage}"))?;
    let bench_dir_name = bench_dir_for_stage(stage)
        .ok_or_else(|| anyhow!("unsupported stage for bench dir: {stage}"))?;
    let mut records = Vec::with_capacity(corpus.datasets.len());

    for dataset in &corpus.datasets {
        let bench_dir = bench_base_dir(out_dir, bench_dir_name, dataset.id);
        let sqlite_path = bench_dir.join("bench.sqlite");
        if !sqlite_path.exists() {
            records.push(BenchResultRecord {
                dataset_id: dataset.id.to_string(),
                tool: tool.to_string(),
                runtime_s: None,
                memory_mb: None,
                exit_code: None,
                metrics: None,
                status: BenchResultStatus::Missing,
            });
            continue;
        }
        let conn = Connection::open(&sqlite_path)
            .with_context(|| format!("open bench sqlite for {}", dataset.id))?;
        let mut stmt = conn.prepare(&format!(
            "SELECT runtime_s, memory_mb, exit_code, metrics_json \
             FROM {table} \
             WHERE tool = ?1 AND input_hash = ?2 \
             ORDER BY record_id DESC, inserted_at DESC LIMIT 1"
        ))?;
        let row = stmt.query_row(params![tool, dataset.sha256_r1], |row| {
            let runtime_s: f64 = row.get(0)?;
            let memory_mb: f64 = row.get(1)?;
            let exit_code: i64 = row.get(2)?;
            let metrics_json: String = row.get(3)?;
            Ok((runtime_s, memory_mb, exit_code, metrics_json))
        });
        match row {
            Ok((runtime_s, memory_mb, exit_code, metrics_json)) => {
                let metrics: JsonValue = serde_json::from_str(&metrics_json)
                    .with_context(|| format!("parse metrics for {}", dataset.id))?;
                let status = if exit_code == 0 {
                    BenchResultStatus::Success
                } else {
                    BenchResultStatus::Failure
                };
                records.push(BenchResultRecord {
                    dataset_id: dataset.id.to_string(),
                    tool: tool.to_string(),
                    runtime_s: Some(runtime_s),
                    memory_mb: Some(memory_mb),
                    exit_code: Some(exit_code),
                    metrics: Some(metrics),
                    status,
                });
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                records.push(BenchResultRecord {
                    dataset_id: dataset.id.to_string(),
                    tool: tool.to_string(),
                    runtime_s: None,
                    memory_mb: None,
                    exit_code: None,
                    metrics: None,
                    status: BenchResultStatus::Missing,
                });
            }
            Err(err) => return Err(err.into()),
        }
    }

    Ok(records)
}
