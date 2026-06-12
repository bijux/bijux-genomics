#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_fastq_tool_serving_map_reports_governed_fastq_stage_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-fastq-tool-serving-map", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_tool_serving_map.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("fastq"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-tool-serving-map.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(27));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.validate_reads")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("observer_specialized_benchmark")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("comparable")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the governed fastqc validation row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.detect_duplicates_premerge")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_execution")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parse_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the governed detect-duplicates-premerge row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.estimate_library_complexity_prealign")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_execution")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parse_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the governed estimate-library-complexity-prealign row"
    );
    for tool_id in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.validate_reads")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("comparable")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed validation row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.detect_adapters")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("observer_specialized_benchmark")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("comparable")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the fixture-backed detect-adapters row for fastqc"
    );
    for tool_id in ["seqfu", "seqkit", "seqkit_stats"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.profile_reads")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed profile-reads row for {tool_id}"
        );
    }
    for tool_id in ["fastp", "prinseq", "seqfu", "seqkit_stats"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.profile_read_lengths")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed profile-read-lengths row for {tool_id}"
        );
    }
    for tool_id in [
        "adapterremoval",
        "alientrimmer",
        "atropos",
        "bbduk",
        "cutadapt",
        "fastp",
        "fastx_clipper",
        "leehom",
        "prinseq",
        "seqkit",
        "skewer",
        "trim_galore",
        "trimmomatic",
    ] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.trim_reads")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed trim-reads row for {tool_id}"
        );
    }
    for tool_id in ["bbduk", "fastp", "prinseq", "seqkit"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.filter_reads")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed filter-reads row for {tool_id}"
        );
    }
    for tool_id in ["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "vsearch"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.merge_pairs")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed merge-pairs row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("umi_tools")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.extract_umis")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the governed extract-umis row for umi_tools"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("cutadapt")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.normalize_primers")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-03-amplicon-mini")
        }),
        "FASTQ readiness map must retain the governed normalize-primers row for cutadapt"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("vsearch")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.remove_chimeras")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-03-amplicon-mini")
        }),
        "FASTQ readiness map must retain the governed remove-chimeras row for vsearch"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("vsearch")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.cluster_otus")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-03-amplicon-mini")
        }),
        "FASTQ readiness map must retain the governed cluster-otus row for vsearch"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.normalize_abundance")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-03-amplicon-mini")
        }),
        "FASTQ readiness map must retain the governed normalize-abundance row for seqkit"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqfu")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.normalize_abundance")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("planned_contract")
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some("declared_only")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("not_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-03-amplicon-mini")
        }),
        "FASTQ readiness map must retain the planned normalize-abundance row for seqfu"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("dada2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.infer_asvs")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_execution")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parse_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-03-amplicon-mini")
        }),
        "FASTQ readiness map must retain the governed infer-asvs row for dada2"
    );
    for (tool_id, stage_id, support_status, adapter_status, parser_status) in [
        (
            "cutadapt",
            "fastq.normalize_primers",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "vsearch",
            "fastq.remove_chimeras",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        ("dada2", "fastq.infer_asvs", "governed_execution", "runnable", "parse_normalized"),
        (
            "vsearch",
            "fastq.cluster_otus",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "seqkit",
            "fastq.normalize_abundance",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "seqfu",
            "fastq.normalize_abundance",
            "planned_contract",
            "declared_only",
            "not_normalized",
        ),
    ] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some(support_status)
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some(adapter_status)
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some(parser_status)
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-03-amplicon-mini")
            }),
            "FASTQ readiness map must retain the governed amplicon row for {stage_id} / {tool_id}"
        );
    }
    for tool_id in ["bayeshammer", "lighter", "musket", "rcorrector"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.correct_errors")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed correct-errors row for {tool_id}"
        );
    }
    for tool_id in ["bowtie2_build", "star"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.index_reference")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("observer_specialized_benchmark")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("comparable")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("asset:reference-index-assets")
            }),
            "FASTQ readiness map must retain the governed index-reference row for {tool_id}"
        );
    }
    for tool_id in ["fastq_scan", "fastqc", "seqkit"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.profile_overrepresented_sequences")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("observer_specialized_benchmark")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("comparable")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed overrepresented-profiling row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("sortmerna")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.deplete_rrna")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the governed deplete-rrna row for sortmerna"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.deplete_host")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the governed deplete-host row for bowtie2"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.deplete_reference_contaminants")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("governed_benchmark_cohort")
                && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_normalized")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-mini")
        }),
        "FASTQ readiness map must retain the governed contaminant-depletion row for bowtie2"
    );
    for tool_id in ["bbduk", "prinseq"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.filter_low_complexity")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed filter-low-complexity row for {tool_id}"
        );
    }
    for tool_id in ["dustmasker", "fastp"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.filter_low_complexity")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("planned_contract")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("declared_only")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("not_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the planned filter-low-complexity row for {tool_id}"
        );
    }
    for tool_id in ["bbduk", "fastp"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.trim_polyg_tails")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed trim-polyg row for {tool_id}"
        );
    }
    for tool_id in ["adapterremoval", "cutadapt", "seqkit"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.trim_terminal_damage")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed trim-terminal-damage row for {tool_id}"
        );
    }
    for tool_id in ["clumpify", "fastuniq"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.remove_duplicates")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-01-mini")
            }),
            "FASTQ readiness map must retain the governed remove-duplicates row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqpurge")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.trim_reads")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("planned_contract")
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some("declared_only")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("not_normalized")
        }),
        "FASTQ readiness map must retain the planned seqpurge trim-reads row"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("diamond")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
        }),
        "FASTQ readiness map must not retain removed diamond taxonomy rows"
    );
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.screen_taxonomy")
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("governed_benchmark_cohort")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("benchmark_normalized")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some("fixture:corpus-02-edna-mini")
            }),
            "FASTQ readiness map must retain the governed taxonomy-screen row for {tool_id}"
        );
    }
    assert_eq!(
        rows.iter()
            .filter(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
            })
            .count(),
        4,
        "FASTQ readiness map must publish exactly the four governed taxonomy rows"
    );
}
