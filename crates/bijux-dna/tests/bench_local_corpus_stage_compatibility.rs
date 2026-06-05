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
fn bench_local_corpus_stage_compatibility_reports_governed_51_stage_slice() {
    let payload =
        run_cli_json(&["bench", "local", "validate-corpus-stage-compatibility", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_corpus_stage_compatibility_validation.v1")
    );
    assert_eq!(
        payload.get("matrix_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/corpus-stage-compatibility.toml")
    );
    assert_eq!(payload.get("fixture_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        payload.get("fixture_backed_stage_count").and_then(serde_json::Value::as_u64),
        Some(46)
    );
    assert_eq!(
        payload.get("planner_only_stage_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let stages = payload.get("stages").and_then(serde_json::Value::as_array).expect("stages array");
    assert_eq!(stages.len(), 51);
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "detect_duplicates_premerge must map to the governed general FASTQ corpus once duplicate-signal coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_adapters")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "detect_adapters must map to the governed general FASTQ corpus once adapter-hit coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_reads")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "filter_reads must map to the governed general FASTQ corpus once filter-signal coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.merge_pairs")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "merge_pairs must map to the governed general FASTQ corpus once merge-overlap coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.estimate_library_complexity_prealign")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "estimate_library_complexity_prealign must map to the governed general FASTQ corpus once duplicate-signal complexity coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_polyg_tails")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "trim_polyg_tails must map to the governed general FASTQ corpus once poly-G coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_terminal_damage")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "trim_terminal_damage must map to the governed general FASTQ corpus once aDNA-like fixture coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.remove_duplicates")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "remove_duplicates must map to the governed general FASTQ corpus once duplicate-signal removal coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_low_complexity")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "filter_low_complexity must map to the governed general FASTQ corpus once low-complexity fixture coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.extract_umis")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "extract_umis must map to the governed general FASTQ corpus once known-UMI coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.normalize_abundance")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-03-amplicon-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "normalize_abundance must map to the governed amplicon corpus once the OTU abundance fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "taxonomy stage must map to the governed eDNA corpus"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.qc_pre must map to the governed BAM corpus once duplicate-flagged multicontig coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.mapping_summary")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.mapping_summary must map to the governed BAM corpus once the partial-mapping fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.recalibration")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.recalibration must map to the governed BAM corpus once low-coverage recalibration and known-sites coverage are owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.sex must map to the governed BAM corpus once XY-autosome coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.haplogroups")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.haplogroups must map to the governed BAM corpus once the Y-panel fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.authenticity")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.authenticity must map to the governed BAM corpus once ancient-like damage coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.bias_mitigation")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.bias_mitigation must map to the governed BAM corpus once the GC-window ladder fixture owns before-and-after bias expectations"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.contamination")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.contamination must map to the governed BAM corpus once the contamination-panel fixture and AF resources are owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.overlap_correction")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.overlap_correction must map to the governed BAM corpus once paired-overlap control coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.endogenous_content")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.endogenous_content must map to the governed BAM corpus once the endogenous partial-mapping fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapq_filter")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.mapq_filter must map to the governed BAM corpus once the MAPQ-threshold ladder fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.filter")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.filter must map to the governed BAM corpus once the mixed-filter fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.coverage must map to the governed BAM corpus once the target-window coverage fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.gc_bias")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.gc_bias must map to the governed BAM corpus once the GC-window ladder fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.insert_size")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.insert_size must map to the governed BAM corpus once the insert-size triplet fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.duplication_metrics")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.duplication_metrics must map to the governed BAM corpus once the duplicate-cluster fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.complexity")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.complexity must map to the governed BAM corpus once the complexity-projection fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.markdup")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.markdup must map to the governed BAM corpus once the duplicate-cluster fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.length_filter")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "bam.length_filter must map to the governed BAM corpus once the length-threshold fixture is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("planner_only")
                && stage.get("fixture_id").is_some_and(serde_json::Value::is_null)
        }),
        "report_qc must stay explicit about its planner-only corpus gap"
    );
}
