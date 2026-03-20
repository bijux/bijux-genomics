use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;

use bijux_dna_core::contract::{BenchResultRecord, BenchResultStatus};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::{BenchCorpus, BenchQueryContext, BenchResultsRepository};

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
    } else if stage
        == &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES
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
    } else if stage
        == &bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_REFERENCE_CONTAMINANTS
    {
        Some("bench_fastq_deplete_reference_contaminants_v1")
    } else if stage == &bijux_dna_domain_fastq::STAGE_DEPLETE_RRNA {
        Some("bench_fastq_deplete_rrna_v1")
    } else if stage == &bijux_dna_domain_fastq::stages::ids::STAGE_INDEX_REFERENCE {
        Some("bench_fastq_index_reference_v1")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::table_for_stage;
    use bijux_dna_domain_fastq::stages::ids::{
        STAGE_CLUSTER_OTUS, STAGE_DEPLETE_HOST, STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
        STAGE_DEPLETE_RRNA, STAGE_DETECT_ADAPTERS, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY,
        STAGE_INDEX_REFERENCE, STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE,
        STAGE_NORMALIZE_PRIMERS, STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
        STAGE_PROFILE_READ_LENGTHS, STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES,
        STAGE_SCREEN_TAXONOMY, STAGE_TRIM_POLYG_TAILS, STAGE_TRIM_TERMINAL_DAMAGE,
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
}

/// Load bench results for a stage/tool across the corpus.
///
/// # Errors
/// Returns an error if the bench database cannot be opened or parsed.
pub fn get_results_from_sqlite(
    stage: &StageId,
    tool: &str,
    corpus: &BenchCorpus,
    _context: &BenchQueryContext,
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
