use std::path::PathBuf;

use bijux_core::selection::{objective_spec, select_stage, BenchResultStatus, Objective};
use bijux_domain_fastq::{get_results, BenchCorpus, BenchCorpusId, BenchDataset};
use bijux_pipelines::fastq::{fastq_default_pipeline_spec, DefaultPipelineOptions};
fn bench_base_dir(out: &std::path::Path, stage: &str, sample_id: &str) -> std::path::PathBuf {
    out.join("artifacts")
        .join("bench")
        .join(stage)
        .join(sample_id)
}
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
             record_id INTEGER PRIMARY KEY AUTOINCREMENT,\
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
             metrics_json TEXT,\
             inserted_at TEXT\
             )"
        ),
        [],
    )?;

    for (idx, (tool, input_hash, runtime_s, memory_mb, exit_code)) in rows.iter().enumerate() {
        conn.execute(
            &format!(
                "INSERT INTO {table} (record_id, tool, tool_version, image_digest, runner, platform, input_hash, parameters_json, runtime_s, memory_mb, exit_code, metrics_json, inserted_at)\
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ),
            params![
                i64::try_from(idx).unwrap_or(i64::MAX) + 1,
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
                "{\"metrics\":{\"delta_metrics\":{\"read_retention\":0.9}}}",
                "2024-01-01T00:00:00Z"
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

    let pipeline = fastq_default_pipeline_spec(DefaultPipelineOptions {
        paired: false,
        enable_merge: false,
        enable_correct: false,
        enable_qc_post: true,
        enable_screen: false,
    });

    for stage in pipeline.stages {
        let tools = vec!["tool_fast".to_string(), "tool_slow".to_string()];
        let mut tool_records = Vec::new();
        for tool in &tools {
            let records = get_results(&stage, tool, &corpus, &temp_root)?;
            tool_records.push((tool.clone(), records));
        }
        if tool_records.iter().all(|(_, records)| {
            records.is_empty()
                || records
                    .iter()
                    .all(|record| matches!(record.status, BenchResultStatus::Missing))
        }) {
            continue;
        }
        let objective = objective_spec(Objective::Speed);
        let selection = select_stage(&stage, &tool_records, &objective, false);
        assert_eq!(selection.selected, Some("tool_fast".to_string()));
    }

    Ok(())
}
