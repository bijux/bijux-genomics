#![cfg(feature = "sqlite")]

use std::path::PathBuf;

use bijux_dna_analyze::load::sqlite::bench_results_fastq::SqliteBenchResultsRepository;
use bijux_dna_core::contract::{objective_spec, select_stage};
use bijux_dna_core::contract::{BenchResultStatus, Objective};
use bijux_dna_domain_fastq::{
    bench_dir_name, STAGE_DETECT_ADAPTERS, STAGE_FILTER_READS, STAGE_PROFILE_READS,
    STAGE_REPORT_QC, STAGE_TRIM_READS, STAGE_VALIDATE_READS,
};
use bijux_dna_domain_fastq::{BenchCorpus, BenchCorpusId, BenchDataset, BenchDatasetScenario};
use bijux_dna_domain_fastq::{BenchQueryContext, BenchResultsRepository};
use rusqlite::{params, Connection};

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
             params_hash TEXT,\
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
                "INSERT INTO {table} (record_id, tool, tool_version, image_digest, runner, platform, input_hash, params_hash, parameters_json, runtime_s, memory_mb, exit_code, metrics_json, inserted_at)\
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
            ),
            params![
                i64::try_from(idx).unwrap_or(i64::MAX) + 1,
                tool,
                "1.0",
                "sha256:deadbeef",
                "docker",
                "test",
                input_hash,
                "params",
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
    let temp = bijux_dna_testkit::tempdir_for("analyze-select-fastq");
    let temp_root = temp.path().to_path_buf();

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
                scientific_scope: "unit_test",
                scenarios: vec![BenchDatasetScenario::SparseEdgeCase],
            },
            BenchDataset {
                id: "DATA_B",
                r1: PathBuf::from("/dev/null"),
                r2: None,
                sha256_r1: "hash_b",
                sha256_r2: None,
                paired: false,
                scientific_scope: "unit_test",
                scenarios: vec![BenchDatasetScenario::SparseEdgeCase],
            },
        ],
    );

    let stages = [
        (STAGE_VALIDATE_READS, "bench_fastq_validate_v1"),
        (STAGE_TRIM_READS, "bench_fastq_trim_v2"),
        (STAGE_FILTER_READS, "bench_fastq_filter_v2"),
        (STAGE_PROFILE_READS, "bench_fastq_stats_v1"),
    ];

    for (stage, table) in stages {
        for dataset in &corpus.datasets {
            let bench_dir_name = bench_dir_name(&stage).unwrap_or("unknown");
            let bench_dir = bijux_dna_infra::bench_base_dir(&temp_root, bench_dir_name, dataset.id);
            bijux_dna_infra::ensure_dir(&bench_dir)?;
            let sqlite_path = bench_dir.join("bench.sqlite");
            create_bench_db(
                &sqlite_path,
                table,
                &[
                    ("tool_fast".to_string(), dataset.sha256_r1.to_string(), 1.0, 100.0, 0),
                    ("tool_slow".to_string(), dataset.sha256_r1.to_string(), 5.0, 50.0, 0),
                ],
            )?;
        }
    }

    let pipeline_stages = vec![
        STAGE_VALIDATE_READS,
        STAGE_DETECT_ADAPTERS,
        STAGE_TRIM_READS,
        STAGE_FILTER_READS,
        STAGE_PROFILE_READS,
        STAGE_REPORT_QC,
    ];

    let repo = SqliteBenchResultsRepository::new(temp_root.clone());
    for stage in pipeline_stages {
        let tools = vec!["tool_fast".to_string(), "tool_slow".to_string()];
        let mut tool_records = Vec::new();
        for tool in &tools {
            let records =
                repo.bench_results(&stage, tool, &corpus, &BenchQueryContext::default())?;
            tool_records.push((tool.clone(), records));
        }
        if tool_records.iter().all(|(_, records): &(String, Vec<_>)| {
            records.is_empty()
                || records.iter().all(|record| matches!(record.status, BenchResultStatus::Missing))
        }) {
            continue;
        }
        let objective = objective_spec(Objective::Speed);
        let selection = select_stage(&stage, &tool_records, &objective, false);
        assert_eq!(selection.selected, Some("tool_fast".to_string()));
    }

    Ok(())
}
