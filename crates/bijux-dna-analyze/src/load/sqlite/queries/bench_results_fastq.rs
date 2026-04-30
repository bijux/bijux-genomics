use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;

use bijux_dna_core::contract::{BenchResultRecord, BenchResultStatus};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::bench_repository::BenchQueryContextMatch;
use bijux_dna_domain_fastq::{
    BenchCorpus, BenchDataset, BenchQueryContext, BenchResultsRepository,
};

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
        stage: &StageId,
        tool: &str,
        corpus: &BenchCorpus,
        context: &BenchQueryContext,
    ) -> Result<Vec<BenchResultRecord>> {
        get_results_from_sqlite(stage, tool, corpus, context, &self.root_dir)
    }
}

fn table_for_stage(stage: &StageId) -> Option<&'static str> {
    if stage == &bijux_dna_domain_fastq::STAGE_VALIDATE_READS {
        Some("bench_fastq_validate_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_DETECT_ADAPTERS {
        Some("bench_fastq_detect_adapters_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_TRIM_READS {
        Some("bench_fastq_trim_v2")
    } else if stage == &bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_POLYG_TAILS {
        Some("bench_fastq_trim_polyg_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_TRIM_TERMINAL_DAMAGE {
        Some("bench_fastq_trim_terminal_damage_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_FILTER_READS {
        Some("bench_fastq_filter_v2")
    } else if stage == &bijux_dna_domain_fastq::STAGE_FILTER_LOW_COMPLEXITY {
        Some("bench_fastq_filter_low_complexity_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_PROFILE_READS {
        Some("bench_fastq_stats_v1")
    } else if stage == &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS {
        Some("bench_fastq_read_lengths_v1")
    } else if stage == &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES
    {
        Some("bench_fastq_overrepresented_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_MERGE_PAIRS {
        Some("bench_fastq_merge_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_CORRECT_ERRORS {
        Some("bench_fastq_correct_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_REMOVE_DUPLICATES {
        Some("bench_fastq_duplicates_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_REMOVE_CHIMERAS {
        Some("bench_fastq_chimeras_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_NORMALIZE_PRIMERS {
        Some("bench_fastq_normalize_primers_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_INFER_ASVS {
        Some("bench_fastq_infer_asvs_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_CLUSTER_OTUS {
        Some("bench_fastq_cluster_otus_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_NORMALIZE_ABUNDANCE {
        Some("bench_fastq_normalize_abundance_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_REPORT_QC {
        Some("bench_fastq_qc_post_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_EXTRACT_UMIS {
        Some("bench_fastq_umi_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_SCREEN_TAXONOMY {
        Some("bench_fastq_screen_v1")
    } else if stage == &bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_HOST {
        Some("bench_fastq_deplete_host_v1")
    } else if stage == &bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_REFERENCE_CONTAMINANTS {
        Some("bench_fastq_deplete_reference_contaminants_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_DEPLETE_RRNA {
        Some("bench_fastq_deplete_rrna_v1")
    } else if stage == &bijux_dna_domain_fastq::stages::ids::STAGE_INDEX_REFERENCE {
        Some("bench_fastq_index_reference_v1")
    } else {
        None
    }
}

fn benchmark_input_hashes(dataset: &BenchDataset) -> [Option<String>; 2] {
    let primary = Some(dataset.sha256_r1.to_string());
    let paired = dataset.sha256_r2.map(|sha256_r2| format!("{}+{sha256_r2}", dataset.sha256_r1));
    [primary, paired]
}

fn missing_bench_record(dataset_id: &str, tool: &str) -> BenchResultRecord {
    BenchResultRecord {
        dataset_id: dataset_id.to_string(),
        tool: tool.to_string(),
        runtime_s: None,
        memory_mb: None,
        exit_code: None,
        metrics: None,
        status: BenchResultStatus::Missing,
    }
}

type BenchCandidate = (f64, f64, i64, String);

fn load_bench_candidate(
    sqlite_path: &Path,
    dataset: &BenchDataset,
    table: &str,
    tool: &str,
    context: &BenchQueryContext,
) -> Result<Option<BenchCandidate>> {
    let conn = Connection::open(sqlite_path)
        .with_context(|| format!("open bench sqlite for {}", dataset.id))?;
    let [primary_input_hash, paired_input_hash] = benchmark_input_hashes(dataset);
    let mut stmt = conn.prepare(&format!(
        "SELECT runtime_s, memory_mb, exit_code, metrics_json, parameters_json \
         FROM {table} \
         WHERE tool = ?1 \
           AND (input_hash = ?2 OR (?3 IS NOT NULL AND input_hash = ?3)) \
           AND (?4 IS NULL OR image_digest = ?4) \
           AND (?5 IS NULL OR params_hash = ?5) \
         ORDER BY record_id DESC, inserted_at DESC"
    ))?;
    let rows = stmt.query_map(
        params![
            tool,
            primary_input_hash,
            paired_input_hash,
            context.image_digest.as_deref(),
            context.params_hash.as_deref()
        ],
        |row| {
            let runtime_s: f64 = row.get(0)?;
            let memory_mb: f64 = row.get(1)?;
            let exit_code: i64 = row.get(2)?;
            let metrics_json: String = row.get(3)?;
            let parameters_json: String = row.get(4)?;
            Ok((runtime_s, memory_mb, exit_code, metrics_json, parameters_json))
        },
    )?;
    let mut legacy_candidate = None;
    let mut exact_candidate = None;
    for row in rows {
        let (runtime_s, memory_mb, exit_code, metrics_json, parameters_json) = row?;
        let parameters: JsonValue = serde_json::from_str(&parameters_json)
            .with_context(|| format!("parse benchmark parameters for {}", dataset.id))?;
        let candidate = (runtime_s, memory_mb, exit_code, metrics_json);
        match context.match_against_parameters(&parameters) {
            BenchQueryContextMatch::Exact => {
                exact_candidate = Some(candidate);
                break;
            }
            BenchQueryContextMatch::LegacyCompatible => {
                legacy_candidate.get_or_insert(candidate);
            }
            BenchQueryContextMatch::NoMatch => {}
        }
    }
    Ok(exact_candidate.or(legacy_candidate))
}

fn bench_record_from_candidate(
    dataset: &BenchDataset,
    tool: &str,
    candidate: BenchCandidate,
) -> Result<BenchResultRecord> {
    let (runtime_s, memory_mb, exit_code, metrics_json) = candidate;
    let metrics: JsonValue = serde_json::from_str(&metrics_json)
        .with_context(|| format!("parse metrics for {}", dataset.id))?;
    let status =
        if exit_code == 0 { BenchResultStatus::Success } else { BenchResultStatus::Failure };
    Ok(BenchResultRecord {
        dataset_id: dataset.id.to_string(),
        tool: tool.to_string(),
        runtime_s: Some(runtime_s),
        memory_mb: Some(memory_mb),
        exit_code: Some(exit_code),
        metrics: Some(metrics),
        status,
    })
}

/// Load bench results for a stage/tool across the corpus.
///
/// # Errors
/// Returns an error if the bench database cannot be opened or parsed.
pub fn get_results_from_sqlite(
    stage: &StageId,
    tool: &str,
    corpus: &BenchCorpus,
    context: &BenchQueryContext,
    out_dir: &Path,
) -> Result<Vec<BenchResultRecord>> {
    let table = table_for_stage(stage)
        .ok_or_else(|| anyhow!("unsupported stage for bench query: {}", stage.as_str()))?;
    let bench_dir_name = bijux_dna_domain_fastq::bench_dir_name(stage)
        .ok_or_else(|| anyhow!("unsupported stage for bench dir: {}", stage.as_str()))?;
    let mut records = Vec::with_capacity(corpus.datasets.len());

    for dataset in &corpus.datasets {
        let bench_dir = bijux_dna_infra::bench_base_dir(out_dir, bench_dir_name, dataset.id);
        let sqlite_path = bench_dir.join("bench.sqlite");
        if !sqlite_path.exists() {
            records.push(missing_bench_record(dataset.id, tool));
            continue;
        }
        match load_bench_candidate(&sqlite_path, dataset, table, tool, context)? {
            Some(candidate) => records.push(bench_record_from_candidate(dataset, tool, candidate)?),
            None => records.push(missing_bench_record(dataset.id, tool)),
        }
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bijux_dna_core::prelude::BenchResultStatus;
    use rusqlite::{params, Connection};

    use super::{get_results_from_sqlite, table_for_stage, SqliteBenchResultsRepository};
    use bijux_dna_domain_fastq::stages::ids::{
        STAGE_CLUSTER_OTUS, STAGE_DEPLETE_HOST, STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
        STAGE_DEPLETE_RRNA, STAGE_DETECT_ADAPTERS, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY,
        STAGE_INDEX_REFERENCE, STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE,
        STAGE_NORMALIZE_PRIMERS, STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
        STAGE_PROFILE_READ_LENGTHS, STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES,
        STAGE_SCREEN_TAXONOMY, STAGE_TRIM_POLYG_TAILS, STAGE_TRIM_TERMINAL_DAMAGE,
    };
    use bijux_dna_domain_fastq::{
        bench_dir_name, BenchCorpus, BenchCorpusId, BenchDataset, BenchQueryContext,
        BenchResultsRepository,
    };
    use bijux_dna_domain_fastq::{
        STAGE_CORRECT_ERRORS, STAGE_FILTER_READS, STAGE_PROFILE_READS, STAGE_REPORT_QC,
        STAGE_TRIM_READS, STAGE_VALIDATE_READS,
    };

    #[test]
    fn benchmarked_fastq_stage_surface_has_sqlite_table_mapping() {
        for stage in [
            &STAGE_VALIDATE_READS,
            &STAGE_DETECT_ADAPTERS,
            &STAGE_TRIM_READS,
            &STAGE_TRIM_POLYG_TAILS,
            &STAGE_TRIM_TERMINAL_DAMAGE,
            &STAGE_FILTER_READS,
            &STAGE_FILTER_LOW_COMPLEXITY,
            &STAGE_PROFILE_READS,
            &STAGE_PROFILE_READ_LENGTHS,
            &STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
            &STAGE_MERGE_PAIRS,
            &STAGE_CORRECT_ERRORS,
            &STAGE_REMOVE_DUPLICATES,
            &STAGE_REMOVE_CHIMERAS,
            &STAGE_NORMALIZE_PRIMERS,
            &STAGE_INFER_ASVS,
            &STAGE_CLUSTER_OTUS,
            &STAGE_NORMALIZE_ABUNDANCE,
            &STAGE_REPORT_QC,
            &STAGE_EXTRACT_UMIS,
            &STAGE_SCREEN_TAXONOMY,
            &STAGE_DEPLETE_HOST,
            &STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
            &STAGE_DEPLETE_RRNA,
            &STAGE_INDEX_REFERENCE,
        ] {
            assert!(
                table_for_stage(stage).is_some(),
                "missing sqlite table mapping for {}",
                stage.as_str()
            );
        }
    }

    #[derive(Debug, Clone)]
    struct BenchRowFixture {
        tool: String,
        input_hash: String,
        image_digest: String,
        params_hash: String,
        parameters_json: String,
        runtime_s: f64,
        memory_mb: f64,
        exit_code: i64,
    }

    fn create_bench_db(
        path: &PathBuf,
        table: &str,
        rows: &[BenchRowFixture],
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

        for (idx, row) in rows.iter().enumerate() {
            conn.execute(
                &format!(
                    "INSERT INTO {table} (record_id, tool, tool_version, image_digest, runner, platform, input_hash, params_hash, parameters_json, runtime_s, memory_mb, exit_code, metrics_json, inserted_at)\
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
                ),
                params![
                    i64::try_from(idx).unwrap_or(i64::MAX) + 1,
                    &row.tool,
                    "1.0",
                    &row.image_digest,
                    "docker",
                    "test",
                    &row.input_hash,
                    &row.params_hash,
                    &row.parameters_json,
                    row.runtime_s,
                    row.memory_mb,
                    row.exit_code,
                    "{\"metrics\":{\"delta_metrics\":{\"read_retention\":0.9}}}",
                    "2024-01-01T00:00:00Z"
                ],
            )?;
        }

        Ok(())
    }

    fn bench_corpus_fixture() -> BenchCorpus {
        BenchCorpus::new(
            BenchCorpusId::Fastq5Set,
            vec![BenchDataset {
                id: "DATA_A",
                r1: PathBuf::from("/dev/null"),
                r2: None,
                sha256_r1: "hash_a",
                sha256_r2: None,
                paired: false,
                scientific_scope: "unit_test",
                scenarios: vec![bijux_dna_domain_fastq::BenchDatasetScenario::SparseEdgeCase],
            }],
        )
    }

    fn bench_paired_corpus_fixture() -> BenchCorpus {
        BenchCorpus::new(
            BenchCorpusId::Fastq5Set,
            vec![BenchDataset {
                id: "DATA_PE",
                r1: PathBuf::from("/dev/null"),
                r2: Some(PathBuf::from("/dev/null")),
                sha256_r1: "hash_r1",
                sha256_r2: Some("hash_r2"),
                paired: true,
                scientific_scope: "unit_test",
                scenarios: vec![bijux_dna_domain_fastq::BenchDatasetScenario::CleanPairedReads],
            }],
        )
    }

    #[test]
    fn sqlite_queries_honor_params_hash_filter() -> Result<(), Box<dyn std::error::Error>> {
        let temp = bijux_dna_testkit::tempdir_for("bench-results-fastq-params-hash");
        let root_dir = temp.path().to_path_buf();
        let corpus = bench_corpus_fixture();
        let bench_dir = bijux_dna_infra::bench_base_dir(
            &root_dir,
            bench_dir_name(&STAGE_TRIM_READS).unwrap_or("unknown"),
            corpus.datasets[0].id,
        );
        bijux_dna_infra::ensure_dir(&bench_dir)?;
        create_bench_db(
            &bench_dir.join("bench.sqlite"),
            "bench_fastq_trim_v2",
            &[
                BenchRowFixture {
                    tool: "fastp".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-a".to_string(),
                    params_hash: "params-a".to_string(),
                    parameters_json: "{}".to_string(),
                    runtime_s: 1.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
                BenchRowFixture {
                    tool: "fastp".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-a".to_string(),
                    params_hash: "params-b".to_string(),
                    parameters_json: "{}".to_string(),
                    runtime_s: 4.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
            ],
        )?;

        let records = get_results_from_sqlite(
            &STAGE_TRIM_READS,
            "fastp",
            &corpus,
            &BenchQueryContext::new().with_params_hash("params-b"),
            &root_dir,
        )?;

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].runtime_s, Some(4.0));
        Ok(())
    }

    #[test]
    fn sqlite_repository_honors_image_digest_filter() -> Result<(), Box<dyn std::error::Error>> {
        let temp = bijux_dna_testkit::tempdir_for("bench-results-fastq-image-digest");
        let root_dir = temp.path().to_path_buf();
        let corpus = bench_corpus_fixture();
        let bench_dir = bijux_dna_infra::bench_base_dir(
            &root_dir,
            bench_dir_name(&STAGE_TRIM_READS).unwrap_or("unknown"),
            corpus.datasets[0].id,
        );
        bijux_dna_infra::ensure_dir(&bench_dir)?;
        create_bench_db(
            &bench_dir.join("bench.sqlite"),
            "bench_fastq_trim_v2",
            &[
                BenchRowFixture {
                    tool: "fastp".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-a".to_string(),
                    params_hash: "params-a".to_string(),
                    parameters_json: "{}".to_string(),
                    runtime_s: 1.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
                BenchRowFixture {
                    tool: "fastp".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-b".to_string(),
                    params_hash: "params-a".to_string(),
                    parameters_json: "{}".to_string(),
                    runtime_s: 3.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
            ],
        )?;

        let repo = SqliteBenchResultsRepository::new(root_dir.clone());
        let records = repo.bench_results(
            &STAGE_TRIM_READS,
            "fastp",
            &corpus,
            &BenchQueryContext::new().with_image_digest("sha256:image-b"),
        )?;

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].runtime_s, Some(3.0));
        Ok(())
    }

    #[test]
    fn sqlite_repository_prefers_exact_bench_query_context_matches(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = bijux_dna_testkit::tempdir_for("bench-results-fastq-query-context");
        let root_dir = temp.path().to_path_buf();
        let corpus = bench_corpus_fixture();
        let bench_dir = bijux_dna_infra::bench_base_dir(
            &root_dir,
            bench_dir_name(&STAGE_TRIM_READS).unwrap_or("unknown"),
            corpus.datasets[0].id,
        );
        bijux_dna_infra::ensure_dir(&bench_dir)?;

        let exact_parameters = BenchQueryContext::new()
            .with_stage_contract_hash("contract-a")
            .with_reference_hash("reference-a")
            .with_bank_hash("adapter_bank", "bank-a")
            .embed_in_parameters(&serde_json::json!({"min_length": 50}));
        let mismatch_parameters = BenchQueryContext::new()
            .with_stage_contract_hash("contract-b")
            .with_reference_hash("reference-b")
            .with_bank_hash("adapter_bank", "bank-b")
            .embed_in_parameters(&serde_json::json!({"min_length": 50}));
        create_bench_db(
            &bench_dir.join("bench.sqlite"),
            "bench_fastq_trim_v2",
            &[
                BenchRowFixture {
                    tool: "fastp".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-a".to_string(),
                    params_hash: "params-a".to_string(),
                    parameters_json: serde_json::to_string(&mismatch_parameters)?,
                    runtime_s: 9.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
                BenchRowFixture {
                    tool: "fastp".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-a".to_string(),
                    params_hash: "params-a".to_string(),
                    parameters_json: serde_json::to_string(&exact_parameters)?,
                    runtime_s: 2.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
            ],
        )?;

        let repo = SqliteBenchResultsRepository::new(root_dir.clone());
        let records = repo.bench_results(
            &STAGE_TRIM_READS,
            "fastp",
            &corpus,
            &BenchQueryContext::new()
                .with_stage_contract_hash("contract-a")
                .with_reference_hash("reference-a")
                .with_bank_hash("adapter_bank", "bank-a"),
        )?;

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].runtime_s, Some(2.0));
        Ok(())
    }

    #[test]
    fn sqlite_repository_falls_back_to_legacy_rows_without_embedded_query_context(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp = bijux_dna_testkit::tempdir_for("bench-results-fastq-legacy-query-context");
        let root_dir = temp.path().to_path_buf();
        let corpus = bench_corpus_fixture();
        let bench_dir = bijux_dna_infra::bench_base_dir(
            &root_dir,
            bench_dir_name(&STAGE_TRIM_READS).unwrap_or("unknown"),
            corpus.datasets[0].id,
        );
        bijux_dna_infra::ensure_dir(&bench_dir)?;
        create_bench_db(
            &bench_dir.join("bench.sqlite"),
            "bench_fastq_trim_v2",
            &[BenchRowFixture {
                tool: "fastp".to_string(),
                input_hash: corpus.datasets[0].sha256_r1.to_string(),
                image_digest: "sha256:image-a".to_string(),
                params_hash: "params-a".to_string(),
                parameters_json: "{}".to_string(),
                runtime_s: 7.0,
                memory_mb: 128.0,
                exit_code: 0,
            }],
        )?;

        let repo = SqliteBenchResultsRepository::new(root_dir.clone());
        let records = repo.bench_results(
            &STAGE_TRIM_READS,
            "fastp",
            &corpus,
            &BenchQueryContext::new().with_stage_contract_hash("contract-a"),
        )?;

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].runtime_s, Some(7.0));
        Ok(())
    }

    #[test]
    fn sqlite_repository_honors_embedded_lineage_hash() -> Result<(), Box<dyn std::error::Error>> {
        let temp = bijux_dna_testkit::tempdir_for("bench-results-fastq-lineage-query-context");
        let root_dir = temp.path().to_path_buf();
        let corpus = bench_corpus_fixture();
        let bench_dir = bijux_dna_infra::bench_base_dir(
            &root_dir,
            bench_dir_name(&STAGE_REPORT_QC).unwrap_or("unknown"),
            corpus.datasets[0].id,
        );
        bijux_dna_infra::ensure_dir(&bench_dir)?;

        let fastp_lineage_hash = format!("{}=fastp", STAGE_TRIM_READS.as_str());
        let bbduk_lineage_hash = format!("{}=bbduk", STAGE_TRIM_READS.as_str());
        let fastp_lineage = BenchQueryContext::new()
            .with_stage_contract_hash("contract-a")
            .with_lineage_hash(&fastp_lineage_hash)
            .embed_in_parameters(&serde_json::json!({}));
        let bbduk_lineage = BenchQueryContext::new()
            .with_stage_contract_hash("contract-a")
            .with_lineage_hash(&bbduk_lineage_hash)
            .embed_in_parameters(&serde_json::json!({}));
        create_bench_db(
            &bench_dir.join("bench.sqlite"),
            "bench_fastq_qc_post_v1",
            &[
                BenchRowFixture {
                    tool: "multiqc".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-a".to_string(),
                    params_hash: "params-a".to_string(),
                    parameters_json: serde_json::to_string(&bbduk_lineage)?,
                    runtime_s: 8.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
                BenchRowFixture {
                    tool: "multiqc".to_string(),
                    input_hash: corpus.datasets[0].sha256_r1.to_string(),
                    image_digest: "sha256:image-a".to_string(),
                    params_hash: "params-a".to_string(),
                    parameters_json: serde_json::to_string(&fastp_lineage)?,
                    runtime_s: 3.0,
                    memory_mb: 128.0,
                    exit_code: 0,
                },
            ],
        )?;

        let repo = SqliteBenchResultsRepository::new(root_dir.clone());
        let records = repo.bench_results(
            &STAGE_REPORT_QC,
            "multiqc",
            &corpus,
            &BenchQueryContext::new()
                .with_stage_contract_hash("contract-a")
                .with_lineage_hash(&fastp_lineage_hash),
        )?;

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].runtime_s, Some(3.0));
        Ok(())
    }

    #[test]
    fn sqlite_repository_matches_paired_input_hashes() -> Result<(), Box<dyn std::error::Error>> {
        let temp = bijux_dna_testkit::tempdir_for("bench-results-fastq-paired-input-hash");
        let root_dir = temp.path().to_path_buf();
        let corpus = bench_paired_corpus_fixture();
        let paired_hash =
            corpus.datasets[0].sha256_r2.ok_or("paired corpus fixture is missing read 2 hash")?;
        let bench_dir = bijux_dna_infra::bench_base_dir(
            &root_dir,
            bench_dir_name(&STAGE_DETECT_ADAPTERS).unwrap_or("unknown"),
            corpus.datasets[0].id,
        );
        bijux_dna_infra::ensure_dir(&bench_dir)?;
        create_bench_db(
            &bench_dir.join("bench.sqlite"),
            "bench_fastq_detect_adapters_v1",
            &[BenchRowFixture {
                tool: "fastqc".to_string(),
                input_hash: format!("{}+{}", corpus.datasets[0].sha256_r1, paired_hash,),
                image_digest: "sha256:image-a".to_string(),
                params_hash: "params-a".to_string(),
                parameters_json: "{}".to_string(),
                runtime_s: 5.0,
                memory_mb: 128.0,
                exit_code: 0,
            }],
        )?;

        let records = get_results_from_sqlite(
            &STAGE_DETECT_ADAPTERS,
            "fastqc",
            &corpus,
            &BenchQueryContext::new(),
            &root_dir,
        )?;

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].runtime_s, Some(5.0));
        assert!(matches!(records[0].status, BenchResultStatus::Success));
        Ok(())
    }
}
