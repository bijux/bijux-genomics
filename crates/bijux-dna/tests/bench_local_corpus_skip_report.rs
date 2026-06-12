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
fn bench_local_corpus_skip_report_writes_governed_skip_manifest() {
    let payload = run_cli_json(&["bench", "local", "render-corpus-skip-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_corpus_skip_report.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/corpus-skip-report.json")
    );
    assert_eq!(payload.get("fixture_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("skip_count").and_then(serde_json::Value::as_u64), Some(343));
    assert_eq!(
        payload.get("asset_backed_stage_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("planner_only_stage_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let skips = payload.get("skips").and_then(serde_json::Value::as_array).expect("skips array");
    let asset_backed_stages = payload
        .get("asset_backed_stages")
        .and_then(serde_json::Value::as_array)
        .expect("asset_backed_stages array");
    assert!(asset_backed_stages.iter().any(|stage| {
        stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.index_reference")
            && stage.get("benchmark_scope_id").and_then(serde_json::Value::as_str)
                == Some("reference-index-assets")
    }));
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-kinship-mini")
        }),
        "fixture-backed bam.kinship skips must name the governed kinship BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-adna-bam-mini")
        }),
        "fixture-backed bam.contamination skips must name the governed aDNA BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.qc_pre skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.recalibration")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.recalibration skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapping_summary")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.mapping_summary skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-adna-bam-mini")
        }),
        "fixture-backed bam.sex skips must name the governed aDNA BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-genotyping-mini")
        }),
        "fixture-backed bam.genotyping skips must name the governed genotyping BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.haplogroups")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-adna-bam-mini")
        }),
        "fixture-backed bam.haplogroups skips must name the governed aDNA BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.authenticity")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-adna-damage-mini")
        }),
        "fixture-backed bam.authenticity skips must name the governed aDNA damage corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.bias_mitigation")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.bias_mitigation skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.overlap_correction")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.overlap_correction skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.endogenous_content")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.endogenous_content skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapq_filter")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.mapq_filter skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.filter")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.filter skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.coverage skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.gc_bias")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.gc_bias skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.insert_size")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.insert_size skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.duplication_metrics")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.duplication_metrics skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.complexity")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.complexity skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.markdup")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.markdup skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.length_filter")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        }),
        "fixture-backed bam.length_filter skips must name the governed BAM corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed detect-duplicates skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
        }),
        "incompatible corpora must name their governed replacement corpus"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.normalize_abundance")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-03-amplicon-mini")
        }),
        "fixture-backed normalize-abundance skips must name the governed amplicon corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed filter-reads skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.merge_pairs")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed merge-pairs skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.estimate_library_complexity_prealign")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed estimate-library-complexity skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_polyg_tails")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed trim-polyg skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_terminal_damage")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed trim-terminal-damage skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.remove_duplicates")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed remove-duplicates skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.extract_umis")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed extract-umis skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_low_complexity")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed filter-low-complexity skips must name the governed FASTQ corpus replacement"
    );

    let planner_only = payload
        .get("planner_only_stages")
        .and_then(serde_json::Value::as_array)
        .expect("planner_only_stages array");
    assert!(
        planner_only.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        }),
        "planner-only stages must stay explicit instead of disappearing"
    );
}
