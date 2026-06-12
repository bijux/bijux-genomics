#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
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
        .args(args)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_fastq_active_stage_tool_matrix_reports_only_active_fastq_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-fastq-active-stage-tool-matrix", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_active_stage_tool_matrix.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/fastq-active-stage-tool-matrix.tsv")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(74));
    assert_eq!(payload.get("retained_stage_count").and_then(serde_json::Value::as_u64), Some(27));
    assert_eq!(payload.get("retained_tool_count").and_then(serde_json::Value::as_u64), Some(44));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(26));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(41));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("removed_stage_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("removed_tool_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        payload.get("removed_from_scope_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/removed-from-scope.tsv")
    );

    let support_status_counts = payload
        .get("support_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("support_status_counts");
    assert_eq!(
        support_status_counts.get("governed_benchmark_cohort").and_then(serde_json::Value::as_u64),
        Some(55)
    );
    assert_eq!(
        support_status_counts.get("governed_execution").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        support_status_counts
            .get("observer_specialized_benchmark")
            .and_then(serde_json::Value::as_u64),
        Some(11)
    );

    let parser_status_counts = payload
        .get("parser_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("parser_status_counts");
    assert_eq!(
        parser_status_counts.get("benchmark_normalized").and_then(serde_json::Value::as_u64),
        Some(55)
    );
    assert_eq!(
        parser_status_counts.get("comparable").and_then(serde_json::Value::as_u64),
        Some(11)
    );
    assert_eq!(
        parser_status_counts.get("parse_normalized").and_then(serde_json::Value::as_u64),
        Some(3)
    );

    let corpus_status_counts = payload
        .get("corpus_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("corpus_status_counts");
    assert_eq!(
        corpus_status_counts
            .get("asset:reference-index-assets")
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        corpus_status_counts.get("fixture:corpus-01-mini").and_then(serde_json::Value::as_u64),
        Some(58)
    );
    assert_eq!(
        corpus_status_counts.get("fixture:corpus-02-edna-mini").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        corpus_status_counts
            .get("fixture:corpus-03-amplicon-mini")
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 69);

    assert!(rows.iter().all(|row| {
        row.get("benchmark_status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                != Some("planned_contract")
            && row.get("adapter_status").and_then(serde_json::Value::as_str)
                != Some("declared_only")
            && row.get("corpus_status").and_then(serde_json::Value::as_str) != Some("planner_only")
            && row.get("scope_proof_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("trimmomatic")
            && row.get("corpus_id").and_then(serde_json::Value::as_str) == Some("corpus-01-mini")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("corpus_only")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("fastq.adapter.trim_reads")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("fastq.parser.trim_reads")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("fastq_trim_reads_v2")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("governed_benchmark_cohort")
            && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
            && row.get("parser_status").and_then(serde_json::Value::as_str)
                == Some("benchmark_normalized")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-mini")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("corpus-02-edna-mini")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("database_artifact_id+taxonomy_database_root")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("fastq.adapter.screen_taxonomy")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("fastq.parser.screen_taxonomy")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("fastq_screen_taxonomy_v1")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("governed_benchmark_cohort")
    }));
    let taxonomy_rows = rows
        .iter()
        .filter(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
        })
        .collect::<Vec<_>>();
    assert_eq!(taxonomy_rows.len(), 4);
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(taxonomy_rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                    == Some("database_artifact_id+taxonomy_database_root")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.infer_asvs")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("dada2")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("governed_execution")
            && row.get("parser_status").and_then(serde_json::Value::as_str)
                == Some("parse_normalized")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-03-amplicon-mini")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("fastq.profile_overrepresented_sequences")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
            && row.get("corpus_id").and_then(serde_json::Value::as_str) == Some("corpus-01-mini")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("corpus_only")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("fastq.adapter.profile_overrepresented_sequences")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("fastq.parser.profile_overrepresented_sequences")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("fastq_profile_overrepresented_sequences_v1")
            && row.get("support_status").and_then(serde_json::Value::as_str)
                == Some("observer_specialized_benchmark")
            && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
            && row.get("parser_status").and_then(serde_json::Value::as_str) == Some("comparable")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-mini")
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.index_reference")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("reference-index-assets")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("reference_fasta+reference_index_output")
            && row.get("parser_status").and_then(serde_json::Value::as_str) == Some("comparable")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("asset:reference-index-assets")
    }));
    assert!(
        rows.iter().all(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) != Some("fastq.report_qc")
                && !(row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.trim_reads")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqpurge"))
                && !(row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.normalize_abundance")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqfu"))
        }),
        "FASTQ active matrix must leave only non-active bindings in removed-from-scope"
    );
}
