use std::path::PathBuf;

use bijux_domain_fastq::pipeline::{
    fastq_default_pipeline, rank_tools_for_stage, BenchCorpus, BenchCorpusId, BenchDataset,
    DefaultPipelineOptions, Objective,
};
use bijux_engine::api::bench_base_dir;
use rusqlite::{params, Connection};
use uuid::Uuid;

fn create_bench_db(
    path: &PathBuf,
    table: &str,
    rows: &[(String, String, f64, f64, i64)],
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table} (\
             tool TEXT,\
             tool_version TEXT,\
             image_digest TEXT,\
             runner TEXT,\
             platform TEXT,\
             input_hash TEXT,\
             parameters_json TEXT,\
             runtime_s REAL,\
             memory_mb REAL,\
             exit_code INTEGER,\
             metrics_json TEXT\
             )"
        ),
        [],
    )?;

    for (tool, input_hash, runtime_s, memory_mb, exit_code) in rows {
        conn.execute(
            &format!(
                "INSERT INTO {table} (tool, tool_version, image_digest, runner, platform, input_hash, parameters_json, runtime_s, memory_mb, exit_code, metrics_json)\
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            ),
            params![
                tool,
                "1.0",
                "sha256:deadbeef",
                "docker",
                "test",
                input_hash,
                "{}",
                runtime_s,
                memory_mb,
                exit_code,
                "{\"metrics\":{\"delta_metrics\":{\"read_retention\":0.9}}}"
            ],
        )?;
    }

    Ok(())
}

#[test]
fn default_route_selects_tools_deterministically() -> Result<(), Box<dyn std::error::Error>> {
    let temp_root = std::env::temp_dir().join(format!("bijux-select-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_root)?;

    let corpus = BenchCorpus::new(
        BenchCorpusId::Fastq5Set,
        vec![
            BenchDataset {
                id: "DATA_A",
                r1: PathBuf::from("/dev/null"),
                r2: None,
                sha256_r1: "hash_a",
                sha256_r2: None,
                paired: false,
            },
            BenchDataset {
                id: "DATA_B",
                r1: PathBuf::from("/dev/null"),
                r2: None,
                sha256_r1: "hash_b",
                sha256_r2: None,
                paired: false,
            },
        ],
    );

    let stages = [
        ("fastq.validate_pre", "bench_fastq_validate_v1"),
        ("fastq.trim", "bench_fastq_trim_v2"),
        ("fastq.filter", "bench_fastq_filter_v2"),
        ("fastq.stats_neutral", "bench_fastq_stats_v1"),
    ];

    for (stage, table) in stages {
        for dataset in &corpus.datasets {
            let bench_dir_name = match stage {
                "fastq.validate_pre" => "validate",
                "fastq.trim" => "trim",
                "fastq.filter" => "filter",
                "fastq.stats_neutral" => "stats",
                _ => "unknown",
            };
            let bench_dir = bench_base_dir(&temp_root, bench_dir_name, dataset.id);
            std::fs::create_dir_all(&bench_dir)?;
            let sqlite_path = bench_dir.join("bench.sqlite");
            create_bench_db(
                &sqlite_path,
                table,
                &[
                    (
                        "tool_fast".to_string(),
                        dataset.sha256_r1.to_string(),
                        1.0,
                        100.0,
                        0,
                    ),
                    (
                        "tool_slow".to_string(),
                        dataset.sha256_r1.to_string(),
                        5.0,
                        50.0,
                        0,
                    ),
                ],
            )?;
        }
    }

    let pipeline = fastq_default_pipeline(DefaultPipelineOptions {
        paired: false,
        enable_merge: false,
        enable_correct: false,
    });

    for stage in pipeline.stages {
        let tools = vec!["tool_fast".to_string(), "tool_slow".to_string()];
        let selection =
            rank_tools_for_stage(&stage, &tools, Objective::Speed, &corpus, &temp_root, false)?;
        assert_eq!(selection.selected, Some("tool_fast".to_string()));
    }

    Ok(())
}
