#![allow(clippy::expect_used)]

use std::path::PathBuf;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

const GOVERNED_COVERAGE_SAMPLE_ID: &str = "human_like_target_window_coverage";

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

fn run_cli_json_with_repo_root(args: &[&str]) -> (PathBuf, serde_json::Value) {
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

    (repo_root, serde_json::from_slice(&output.stdout).expect("parse stdout as json"))
}

#[test]
fn bench_local_stage_inventory_fastq_json_reports_governed_27_stage_slice() {
    let payload = run_cli_json(&["bench", "local", "list-stages", "--domain", "fastq", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_inventory.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("fastq"));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(27));
    assert_eq!(
        payload.get("stages").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(27)
    );
}

#[test]
fn bench_local_stage_inventory_bam_json_reports_governed_24_stage_slice() {
    let payload = run_cli_json(&["bench", "local", "list-stages", "--domain", "bam", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_inventory.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("bam"));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(
        payload.get("stages").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(24)
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_stage_commands_json_reports_governed_51_command_slice() {
    let payload = run_cli_json(&["bench", "local", "render-stage-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_commands.v3")
    );
    assert_eq!(
        payload.get("script_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/rendered-stage-commands.sh")
    );
    assert_eq!(
        payload.get("manifest_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/rendered-stage-commands.json")
    );
    assert_eq!(
        payload.get("argv_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/rendered-stage-commands.argv.jsonl")
    );
    assert_eq!(payload.get("command_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        payload.get("commands").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(51)
    );
    assert!(
        payload
            .get("commands")
            .and_then(serde_json::Value::as_array)
            .expect("commands array")
            .iter()
            .all(|entry| {
                entry.get("stage_id").and_then(serde_json::Value::as_str).is_some()
                    && entry.get("tool_id").and_then(serde_json::Value::as_str).is_some()
                    && entry.get("threads").and_then(serde_json::Value::as_u64).is_some()
                    && entry.get("memory_mb").and_then(serde_json::Value::as_u64).is_some()
                    && entry.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                        argv.first().and_then(serde_json::Value::as_str) == Some("cargo")
                            && argv.iter().any(|arg| arg.as_str() == Some("--stage-id"))
                    })
                    && entry.get("command").and_then(serde_json::Value::as_str).is_some()
                    && entry
                        .get("inputs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|inputs| !inputs.is_empty())
                    && entry
                        .get("outputs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|outputs| !outputs.is_empty())
            }),
        "every rendered command row must carry tool, IO, resource, and command fields"
    );
    assert!(
        payload
            .get("commands")
            .and_then(serde_json::Value::as_array)
            .expect("commands array")
            .iter()
            .any(|entry| {
                entry.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
            }),
        "rendered command inventory must include the governed report_qc stage"
    );
}

#[test]
fn bench_local_materialize_stage_report_qc_json_writes_governed_smoke_bundle() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "fastq.report_qc",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.report_qc")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/fastq.report_qc/report.json")
    );
}

#[test]
fn bench_local_materialize_stage_bam_validate_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.validate",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.validate"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.validate/validation.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let summary: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.validate validation summary"),
    )
    .expect("parse bam.validate validation summary");

    assert_eq!(
        summary.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.validate.local_smoke.report.v1")
    );
    assert_eq!(summary.get("case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(summary.get("all_cases_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert!(
        summary.get("cases").and_then(serde_json::Value::as_array).is_some_and(|cases| {
            cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("core-v1-coordinate-pass")
                    && case.get("validation_status").and_then(serde_json::Value::as_str)
                        == Some("pass")
            }) && cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("core-v1-malformed-refusal")
                    && case.get("validation_status").and_then(serde_json::Value::as_str)
                        == Some("refusal")
                    && case.get("refusal_codes").and_then(serde_json::Value::as_array).is_some_and(
                        |codes| codes.contains(&serde_json::json!("malformed_alignment_record")),
                    )
            })
        }),
        "bam.validate local smoke summary must cover governed pass and refusal cases"
    );
}

#[test]
fn bench_local_materialize_stage_bam_qc_pre_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.qc_pre",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.qc_pre"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.qc_pre/qc_pre.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let summary: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.qc_pre summary"),
    )
    .expect("parse bam.qc_pre summary");

    assert_eq!(
        summary.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.qc_pre.local_smoke.report.v1")
    );
    assert_eq!(summary.get("case_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(summary.get("all_cases_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert!(
        summary.get("cases").and_then(serde_json::Value::as_array).is_some_and(|cases| {
            cases.len() == 1
                && cases.iter().any(|case| {
                    case.get("sample_id").and_then(serde_json::Value::as_str)
                        == Some("human_like_duplicate_flagged_multicontig")
                        && case.get("total_reads").and_then(serde_json::Value::as_u64) == Some(3)
                        && case.get("mapped_reads").and_then(serde_json::Value::as_u64) == Some(3)
                        && case.get("unmapped_reads").and_then(serde_json::Value::as_u64) == Some(0)
                        && case.get("duplicate_flagged_reads").and_then(serde_json::Value::as_u64)
                            == Some(1)
                        && case
                            .get("contig_summary")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|contigs| {
                                contigs
                                    == &vec![
                                        serde_json::json!({
                                            "contig": "chr1",
                                            "length": 100,
                                            "mapped": 2,
                                            "unmapped": 0
                                        }),
                                        serde_json::json!({
                                            "contig": "chr2",
                                            "length": 80,
                                            "mapped": 1,
                                            "unmapped": 0
                                        }),
                                    ]
                            })
                })
        }),
        "bam.qc_pre local smoke summary must preserve the governed count and contig contract"
    );
}

#[test]
fn bench_local_materialize_stage_bam_mapping_summary_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.mapping_summary",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.mapping_summary")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.mapping_summary/mapping_summary.tsv")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let body = std::fs::read_to_string(&artifact_path).expect("read bam.mapping_summary TSV");
    let mut lines = body.lines();
    let header = lines.next().expect("mapping_summary header");
    assert_eq!(
        header,
        "sample_id\ttotal_reads\tmapped_reads\tunmapped_reads\tmapping_fraction\tsecondary_reads\tsupplementary_reads\treference_name\texpectation_matched\tmapping_summary_json\tflagstat\tidxstats\tstats\tstage_metrics"
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "mapping_summary smoke bundle must contain exactly one row");
    assert_eq!(
        rows[0],
        "human_like_partial_mapping\t3\t2\t1\t0.666667\t0\t0\tchr1\ttrue\ttarget/local-smoke/bam.mapping_summary/human_like_partial_mapping/samtools/mapping.summary.json\ttarget/local-smoke/bam.mapping_summary/human_like_partial_mapping/samtools/flagstat.txt\ttarget/local-smoke/bam.mapping_summary/human_like_partial_mapping/samtools/idxstats.txt\ttarget/local-smoke/bam.mapping_summary/human_like_partial_mapping/samtools/samtools_stats.txt\ttarget/local-smoke/bam.mapping_summary/human_like_partial_mapping/samtools/stage.metrics.json"
    );
}

#[test]
fn bench_local_materialize_stage_bam_filter_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.filter",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.filter"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.filter/filter_metrics.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let metrics: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.filter metrics"),
    )
    .expect("parse bam.filter metrics");

    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.filter.local_smoke.metrics.v1")
    );
    assert_eq!(
        metrics.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_mixed_filter_constraints")
    );
    assert_eq!(metrics.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(metrics.get("input_reads").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(metrics.get("kept_reads").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("removed_reads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(
        metrics.get("active_filters"),
        Some(&serde_json::json!([
            "mapq_threshold",
            "exclude_flags",
            "min_length",
            "remove_duplicates"
        ]))
    );

    let filtered_bam = repo_root.join(
        metrics.get("filtered_bam").and_then(serde_json::Value::as_str).expect("filtered bam path"),
    );
    let filtered_bai = repo_root.join(
        metrics.get("filtered_bai").and_then(serde_json::Value::as_str).expect("filtered bai path"),
    );
    assert!(filtered_bai.is_file(), "bam.filter smoke bundle must expose the curated BAI");

    let filtered_body = std::fs::read_to_string(&filtered_bam).expect("read filtered bam");
    assert!(filtered_body.contains("good001"));
    assert!(!filtered_body.contains("lowq001"));
    assert!(!filtered_body.contains("short001"));
    assert!(!filtered_body.contains("dup001"));
    assert!(!filtered_body.contains("unmap001"));
}

#[test]
fn bench_local_materialize_stage_bam_mapq_filter_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.mapq_filter",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.mapq_filter")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.mapq_filter/mapq_filter.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.mapq_filter report"),
    )
    .expect("parse bam.mapq_filter report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.mapq_filter.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_mapq_threshold_ladder")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("mapq_threshold").and_then(serde_json::Value::as_u64), Some(30));
    assert_eq!(report.get("input_reads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(report.get("kept_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("removed_reads").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("mapped_reads_removed").and_then(serde_json::Value::as_u64), Some(1));

    let filtered_bam = repo_root.join(
        report.get("filtered_bam").and_then(serde_json::Value::as_str).expect("filtered bam path"),
    );
    let filtered_bai = repo_root.join(
        report.get("filtered_bai").and_then(serde_json::Value::as_str).expect("filtered bai path"),
    );
    assert!(filtered_bai.is_file(), "bam.mapq_filter smoke bundle must expose the curated BAI");

    let filtered_body = std::fs::read_to_string(&filtered_bam).expect("read mapq filtered bam");
    assert!(filtered_body.contains("mapq60"));
    assert!(filtered_body.contains("mapq30"));
    assert!(filtered_body.contains("unmapped"));
    assert!(!filtered_body.contains("mapq10"));
}

#[test]
fn bench_local_materialize_stage_bam_length_filter_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.length_filter",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.length_filter")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.length_filter/length_filter.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.length_filter report"),
    )
    .expect("parse bam.length_filter report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.length_filter.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_length_threshold_ladder")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("min_length_threshold").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(report.get("input_reads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(report.get("kept_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("removed_reads").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("observed_min_length").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(report.get("observed_max_length").and_then(serde_json::Value::as_u64), Some(12));

    let filtered_bam = repo_root.join(
        report.get("filtered_bam").and_then(serde_json::Value::as_str).expect("filtered bam path"),
    );
    let filtered_bai = repo_root.join(
        report.get("filtered_bai").and_then(serde_json::Value::as_str).expect("filtered bai path"),
    );
    assert!(filtered_bai.is_file(), "bam.length_filter smoke bundle must expose the curated BAI");

    let filtered_body = std::fs::read_to_string(&filtered_bam).expect("read length filtered bam");
    assert!(filtered_body.contains("len12"));
    assert!(filtered_body.contains("len8"));
    assert!(filtered_body.contains("unmapped10"));
    assert!(!filtered_body.contains("len5"));
}

#[test]
fn bench_local_materialize_stage_bam_markdup_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.markdup",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.markdup"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.markdup/duplicates.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.markdup report"),
    )
    .expect("parse bam.markdup report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.markdup.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_duplicate_cluster")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("duplicate_action").and_then(serde_json::Value::as_str), Some("mark"));
    assert_eq!(report.get("input_reads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(report.get("output_reads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(report.get("removed_reads").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(report.get("duplicate_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("duplicate_reads_after").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("newly_marked_reads").and_then(serde_json::Value::as_u64), Some(1));

    let marked_bam = repo_root.join(
        report.get("marked_bam").and_then(serde_json::Value::as_str).expect("marked bam path"),
    );
    let marked_bai = repo_root.join(
        report.get("marked_bai").and_then(serde_json::Value::as_str).expect("marked bai path"),
    );
    assert!(marked_bai.is_file(), "bam.markdup smoke bundle must expose the curated BAI");

    let marked_body = std::fs::read_to_string(&marked_bam).expect("read marked bam");
    assert!(marked_body.contains("dup_primary"));
    assert!(marked_body.contains("dup_copy\t1024\tchr1\t5"));
    assert!(marked_body.contains("unique_support"));
    assert!(marked_body.contains("unmapped_support"));
}

#[test]
fn bench_local_materialize_stage_bam_duplication_metrics_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.duplication_metrics",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.duplication_metrics")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.duplication_metrics/duplication_metrics.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.duplication_metrics report"),
    )
    .expect("parse bam.duplication_metrics report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.duplication_metrics.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_duplicate_cluster")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("examined_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("duplicate_reads").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("duplicate_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        report.get("duplicate_fraction").and_then(serde_json::Value::as_f64),
        Some(1.0 / 3.0)
    );
    assert_eq!(report.get("estimated_library_size"), Some(&serde_json::Value::Null));
    assert_eq!(
        report.get("insufficient_library_size_reason").and_then(serde_json::Value::as_str),
        Some("tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate")
    );

    let duplication_summary = repo_root.join(
        report
            .get("duplication_summary")
            .and_then(serde_json::Value::as_str)
            .expect("duplication summary path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        duplication_summary.is_file(),
        "bam.duplication_metrics smoke bundle must expose the governed duplication summary"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.duplication_metrics smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_complexity_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.complexity",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.complexity"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.complexity/complexity.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.complexity report"),
    )
    .expect("parse bam.complexity report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.complexity.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_complexity_projection")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("observed_total_reads").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(report.get("observed_unique_reads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(report.get("estimated_unique_reads").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(report.get("estimated_library_size").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(
        report.get("saturation_estimate").and_then(serde_json::Value::as_f64),
        Some(0.33333333333333337_f64)
    );
    assert_eq!(report.get("insufficient_data_reason"), Some(&serde_json::Value::Null));

    let complexity_curve = repo_root.join(
        report
            .get("complexity_curve")
            .and_then(serde_json::Value::as_str)
            .expect("complexity curve path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        complexity_curve.is_file(),
        "bam.complexity smoke bundle must expose the governed complexity curve"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.complexity smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_coverage_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.coverage",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.coverage"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.coverage/coverage.tsv")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let coverage_tsv = std::fs::read_to_string(&artifact_path).expect("read bam.coverage summary");
    assert!(coverage_tsv.contains("sample_id\tregion_id\tcontig"));
    assert!(coverage_tsv.contains(&format!("{GOVERNED_COVERAGE_SAMPLE_ID}\tchr1_window\tchr1")));
    assert!(coverage_tsv.contains(&format!("{GOVERNED_COVERAGE_SAMPLE_ID}\tchr2_window\tchr2")));
    assert!(coverage_tsv.contains("low_pass\ttrue\ttrue"));

    let case_summary = repo_root.join(
        "target/local-smoke/bam.coverage/human_like_target_window_coverage/samtools/coverage.summary.json",
    );
    let stage_metrics = repo_root
        .join("target/local-smoke/bam.coverage/human_like_target_window_coverage/samtools/stage.metrics.json");
    assert!(
        case_summary.is_file(),
        "bam.coverage smoke bundle must expose the governed coverage summary json"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.coverage smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_insert_size_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.insert_size",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.insert_size")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.insert_size/insert_size.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.insert_size report"),
    )
    .expect("parse bam.insert_size report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.insert_size.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_insert_size_triplet")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("read_pairs").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("median_insert_size").and_then(serde_json::Value::as_f64), Some(20.0));
    assert_eq!(
        report.get("mean_insert_size").and_then(serde_json::Value::as_f64),
        Some(21.666666666666668)
    );
    assert_eq!(report.get("min_insert_size").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(report.get("max_insert_size").and_then(serde_json::Value::as_u64), Some(30));
    assert_eq!(report.get("insufficient_pairs_reason"), Some(&serde_json::Value::Null));

    let insert_size_summary = repo_root.join(
        report
            .get("insert_size_summary")
            .and_then(serde_json::Value::as_str)
            .expect("insert-size summary path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        insert_size_summary.is_file(),
        "bam.insert_size smoke bundle must expose the governed insert-size summary"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.insert_size smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_gc_bias_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.gc_bias",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.gc_bias"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.gc_bias/gc_bias.tsv")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let gc_bias_tsv = std::fs::read_to_string(&artifact_path).expect("read bam.gc_bias summary");
    assert!(gc_bias_tsv.contains("sample_id\tgc_bin\tnormalized_coverage\twindows\tread_starts"));
    assert!(gc_bias_tsv.contains("human_like_gc_window_ladder\t0\t0.750000\t1\t1"));
    assert!(gc_bias_tsv.contains("human_like_gc_window_ladder\t50\t1.500000\t1\t2"));
    assert!(gc_bias_tsv.contains("human_like_gc_window_ladder\t100\t0.750000\t1\t1"));

    let gc_bias_summary = repo_root.join(
        "target/local-smoke/bam.gc_bias/human_like_gc_window_ladder/picard/gc_bias.summary.json",
    );
    let stage_metrics = repo_root.join(
        "target/local-smoke/bam.gc_bias/human_like_gc_window_ladder/picard/stage.metrics.json",
    );
    assert!(
        gc_bias_summary.is_file(),
        "bam.gc_bias smoke bundle must expose the governed gc-bias summary json"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.gc_bias smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_endogenous_content_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.endogenous_content",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.endogenous_content")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.endogenous_content/endogenous_content.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.endogenous_content report"),
    )
    .expect("parse bam.endogenous_content report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.endogenous_content.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_endogenous_partial_mapping")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        report.get("method").and_then(serde_json::Value::as_str),
        Some("mapped_fraction_from_flagstat")
    );
    assert_eq!(
        report.get("host_reference_scope").and_then(serde_json::Value::as_str),
        Some("human_host")
    );
    assert_eq!(report.get("mapped_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("endogenous_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("total_reads").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(report.get("endogenous_fraction").and_then(serde_json::Value::as_f64), Some(0.6));
    assert_eq!(report.get("prealignment_fraction"), Some(&serde_json::Value::Null));

    let endogenous_report = repo_root.join(
        report
            .get("endogenous_report")
            .and_then(serde_json::Value::as_str)
            .expect("endogenous report path"),
    );
    let endogenous_summary = repo_root.join(
        report
            .get("endogenous_summary")
            .and_then(serde_json::Value::as_str)
            .expect("endogenous summary path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        endogenous_report.is_file(),
        "bam.endogenous_content smoke bundle must expose the governed endogenous-content report"
    );
    assert!(
        endogenous_summary.is_file(),
        "bam.endogenous_content smoke bundle must expose the governed endogenous-content summary"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.endogenous_content smoke bundle must expose the governed stage metrics"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_materialize_stage_bam_bias_mitigation_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.bias_mitigation",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.bias_mitigation")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.bias_mitigation/bias_mitigation.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.bias_mitigation report"),
    )
    .expect("parse bam.bias_mitigation report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.bias_mitigation.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_gc_window_ladder")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("method").and_then(serde_json::Value::as_str), Some("mapdamage2"));
    assert_eq!(
        report.get("metric_name").and_then(serde_json::Value::as_str),
        Some("gc_bias_score")
    );
    assert_eq!(report.get("pre_mitigation_metric").and_then(serde_json::Value::as_f64), Some(0.25));
    assert_eq!(
        report.get("post_mitigation_metric").and_then(serde_json::Value::as_f64),
        Some(0.125)
    );
    assert_eq!(report.get("metric_delta").and_then(serde_json::Value::as_f64), Some(0.125));
    assert_eq!(
        report.get("mitigation_projection_basis").and_then(serde_json::Value::as_str),
        Some("policy_projection")
    );
    assert_eq!(report.get("mitigation_actions"), Some(&serde_json::json!(["gc_bias_correction"])));
    assert_eq!(report.get("consumed_metrics"), Some(&serde_json::json!(["gc_bias_score"])));

    let bias_report = repo_root.join(
        report.get("bias_report").and_then(serde_json::Value::as_str).expect("bias report path"),
    );
    let bias_summary = repo_root.join(
        report.get("bias_summary").and_then(serde_json::Value::as_str).expect("bias summary path"),
    );
    let bias_policy = repo_root.join(
        report.get("bias_policy").and_then(serde_json::Value::as_str).expect("bias policy path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        bias_report.is_file(),
        "bam.bias_mitigation smoke bundle must expose the governed bias report"
    );
    assert!(
        bias_summary.is_file(),
        "bam.bias_mitigation smoke bundle must expose the governed bias summary"
    );
    assert!(
        bias_policy.is_file(),
        "bam.bias_mitigation smoke bundle must expose the governed bias policy"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.bias_mitigation smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_recalibration_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.recalibration",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.recalibration")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.recalibration/recalibration.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.recalibration report"),
    )
    .expect("parse bam.recalibration report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.recalibration.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("core-v1-recalibration-low-coverage-skip")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("requested_mode").and_then(serde_json::Value::as_str), Some("standard"));
    assert_eq!(report.get("effective_mode").and_then(serde_json::Value::as_str), Some("skip"));
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("skipped"));
    assert_eq!(
        report.get("reason").and_then(serde_json::Value::as_str),
        Some("coverage_below_gate")
    );
    assert_eq!(
        report.get("known_sites"),
        Some(&serde_json::json!(["assets/toy/core-v1/vcf/recalibration_known_sites.vcf"]))
    );
    assert_eq!(
        report.get("coverage_gate"),
        Some(&serde_json::json!({
            "min_mean_coverage": 0.1,
            "min_breadth_1x": 0.05
        }))
    );
    assert_eq!(
        report.get("observed_mean_coverage").and_then(serde_json::Value::as_f64),
        Some(0.024)
    );
    assert_eq!(report.get("observed_breadth_1x").and_then(serde_json::Value::as_f64), Some(0.024));
    assert_eq!(report.get("output_bam_present").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        report.get("recalibration_report_present").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let recalibrated_bam = repo_root.join(
        report
            .get("recalibrated_bam")
            .and_then(serde_json::Value::as_str)
            .expect("recalibrated bam path"),
    );
    let recalibration_report = repo_root.join(
        report
            .get("recalibration_report")
            .and_then(serde_json::Value::as_str)
            .expect("recalibration report path"),
    );
    let recalibration_summary = repo_root.join(
        report
            .get("recalibration_summary")
            .and_then(serde_json::Value::as_str)
            .expect("recalibration summary path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        recalibrated_bam.is_file(),
        "bam.recalibration smoke bundle must expose the governed recalibrated BAM"
    );
    assert!(
        recalibration_report.is_file(),
        "bam.recalibration smoke bundle must expose the governed recalibration report"
    );
    assert!(
        recalibration_summary.is_file(),
        "bam.recalibration smoke bundle must expose the governed recalibration summary"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.recalibration smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_sex_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.sex",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.sex"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.sex/sex.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.sex report"),
    )
    .expect("parse bam.sex report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.sex.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_xy_autosome_coverage")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("method").and_then(serde_json::Value::as_str), Some("rxy"));
    assert_eq!(report.get("chromosome_system").and_then(serde_json::Value::as_str), Some("xy"));
    assert_eq!(report.get("minimum_y_sites").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(report.get("x_coverage").and_then(serde_json::Value::as_f64), Some(0.5));
    assert_eq!(report.get("y_coverage").and_then(serde_json::Value::as_f64), Some(0.5));
    assert_eq!(report.get("autosomal_coverage").and_then(serde_json::Value::as_f64), Some(1.0));
    assert_eq!(report.get("x_to_y_ratio").and_then(serde_json::Value::as_f64), Some(1.0));
    assert_eq!(report.get("call").and_then(serde_json::Value::as_str), Some("male"));
    assert_eq!(report.get("confidence").and_then(serde_json::Value::as_f64), Some(0.9));
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("ok"));
    assert_eq!(report.get("insufficiency_reason"), Some(&serde_json::Value::Null));

    let sex_report = repo_root.join(
        report.get("sex_report").and_then(serde_json::Value::as_str).expect("sex report path"),
    );
    let sex_summary = repo_root.join(
        report.get("sex_summary").and_then(serde_json::Value::as_str).expect("sex summary path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(sex_report.is_file(), "bam.sex smoke bundle must expose the governed sex report");
    assert!(sex_summary.is_file(), "bam.sex smoke bundle must expose the governed sex summary");
    assert!(stage_metrics.is_file(), "bam.sex smoke bundle must expose the governed stage metrics");
}

#[test]
fn bench_local_materialize_stage_bam_overlap_correction_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.overlap_correction",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.overlap_correction")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.overlap_correction/overlap_correction.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.overlap_correction report"),
    )
    .expect("parse bam.overlap_correction report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.overlap_correction.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_paired_overlap_control")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("method").and_then(serde_json::Value::as_str), Some("bamutil"));
    assert_eq!(report.get("pair_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(report.get("corrected_pairs").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("corrected_overlap_bases").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(report.get("insufficiency_reason"), Some(&serde_json::Value::Null));

    let corrected_bam = repo_root.join(
        report
            .get("overlap_corrected_bam")
            .and_then(serde_json::Value::as_str)
            .expect("corrected bam path"),
    );
    let overlap_summary = repo_root.join(
        report
            .get("overlap_correction_summary")
            .and_then(serde_json::Value::as_str)
            .expect("overlap summary path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        corrected_bam.is_file(),
        "bam.overlap_correction smoke bundle must expose the governed corrected BAM"
    );
    assert!(
        overlap_summary.is_file(),
        "bam.overlap_correction smoke bundle must expose the governed overlap summary"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.overlap_correction smoke bundle must expose the governed stage metrics"
    );
}

#[test]
fn bench_local_materialize_stage_bam_authenticity_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.authenticity",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.authenticity")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.authenticity/authenticity.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.authenticity report"),
    )
    .expect("parse bam.authenticity report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.authenticity.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("adna_like_damage")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("method").and_then(serde_json::Value::as_str), Some("authenticct"));
    assert_eq!(
        report.get("consumed_metrics"),
        Some(&serde_json::json!(["damage", "contamination", "complexity", "coverage", "mapping"]))
    );
    assert_eq!(report.get("missing_metrics"), Some(&serde_json::json!([])));

    let authenticity_report = repo_root.join(
        report
            .get("authenticity_report")
            .and_then(serde_json::Value::as_str)
            .expect("authenticity report path"),
    );
    let authenticity_summary = repo_root.join(
        report
            .get("authenticity_summary")
            .and_then(serde_json::Value::as_str)
            .expect("authenticity summary path"),
    );
    let authenticity_composite = repo_root.join(
        report
            .get("authenticity_composite")
            .and_then(serde_json::Value::as_str)
            .expect("authenticity composite path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    let damage_unified_metrics = repo_root.join(
        report
            .get("damage_unified_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("damage unified metrics path"),
    );
    let contamination_summary = repo_root.join(
        report
            .get("contamination_summary")
            .and_then(serde_json::Value::as_str)
            .expect("contamination summary path"),
    );
    let complexity_summary = repo_root.join(
        report
            .get("complexity_summary")
            .and_then(serde_json::Value::as_str)
            .expect("complexity summary path"),
    );
    let coverage_regime = repo_root.join(
        report
            .get("coverage_regime")
            .and_then(serde_json::Value::as_str)
            .expect("coverage regime path"),
    );
    let mapping_summary = repo_root.join(
        report
            .get("mapping_summary")
            .and_then(serde_json::Value::as_str)
            .expect("mapping summary path"),
    );
    for path in [
        &authenticity_report,
        &authenticity_summary,
        &authenticity_composite,
        &stage_metrics,
        &damage_unified_metrics,
        &contamination_summary,
        &complexity_summary,
        &coverage_regime,
        &mapping_summary,
    ] {
        assert!(
            path.is_file(),
            "bam.authenticity smoke bundle must expose the governed composition artifact: {}",
            path.display()
        );
    }
}

#[test]
fn bench_local_materialize_stage_bam_contamination_json_writes_governed_plan_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.contamination",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.contamination")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/bam.contamination/plan.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let plan: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.contamination plan"),
    )
    .expect("parse bam.contamination plan");

    assert_eq!(plan.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.contamination"));
    assert_eq!(plan.get("tool_id").and_then(serde_json::Value::as_str), Some("verifybamid2"));
    assert_eq!(
        plan.get("out_dir").and_then(serde_json::Value::as_str),
        Some("target/local-ready/bam.contamination")
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("sample_id")),
        Some(&serde_json::json!("human_like_contamination_panel_screen"))
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("scope")),
        Some(&serde_json::json!("nuclear"))
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("reference_panels")),
        Some(&serde_json::json!([
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat"
        ]))
    );

    let contamination_report = plan["io"]["outputs"]
        .as_array()
        .and_then(|outputs| {
            outputs
                .iter()
                .find(|artifact| artifact["name"] == serde_json::json!("contamination_report"))
        })
        .unwrap_or_else(|| {
            panic!("contamination_report output missing from CLI materialized plan")
        });
    assert_eq!(
        contamination_report["path"],
        serde_json::json!("target/local-ready/bam.contamination/contamination.json")
    );
    assert!(
        plan["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam.bai"
                ) && shell.contains(
                    "tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
                ) && shell.contains(
                    "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat"
                )
                    && shell.contains("target/local-ready/bam.contamination/contamination.summary.json")
            })
        )),
        "CLI materialized contamination plan must carry the governed BAI, reference, panel, and summary output paths"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_materialize_stage_bam_haplogroups_json_writes_governed_plan_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.haplogroups",
        "--json",
    ]);

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.haplogroups")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/bam.haplogroups/plan.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let plan: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.haplogroups plan"),
    )
    .expect("parse bam.haplogroups plan");

    assert_eq!(plan.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.haplogroups"));
    assert_eq!(plan.get("tool_id").and_then(serde_json::Value::as_str), Some("yleaf"));
    assert_eq!(
        plan.get("out_dir").and_then(serde_json::Value::as_str),
        Some("target/local-ready/bam.haplogroups")
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("sample_id")),
        Some(&serde_json::json!("core-v1-haplogroups-y-panel-screen"))
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("reference_panel_id")),
        Some(&serde_json::json!("toy-human-y-hg38"))
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("coverage_gate")),
        Some(&serde_json::json!({ "min_coverage": 2.0 }))
    );

    let haplogroups_report = plan["io"]["outputs"]
        .as_array()
        .and_then(|outputs| {
            outputs.iter().find(|artifact| artifact["name"] == serde_json::json!("haplogroups"))
        })
        .unwrap_or_else(|| panic!("haplogroups output missing from CLI materialized plan"));
    assert_eq!(
        haplogroups_report["path"],
        serde_json::json!("target/local-ready/bam.haplogroups/haplogroups.json")
    );
    assert!(
        plan["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam")
                    && shell.contains("assets/toy/core-v1/bam/haplogroups_y_panel_screen.sam.bai")
                    && shell.contains("assets/reference/host/references/toy_human_y_haplogroup_panel.tsv")
                    && shell.contains("--reference_genome hg38")
                    && shell.contains("target/local-ready/bam.haplogroups/haplogroups")
            })
        )),
        "CLI materialized haplogroups plan must carry the governed BAM, BAI, panel, reference build, and assignment output prefix"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_materialize_stage_bam_genotyping_json_writes_governed_plan_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.genotyping",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.genotyping"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/bam.genotyping/plan.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let plan: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.genotyping plan"),
    )
    .expect("parse bam.genotyping plan");

    assert_eq!(plan.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.genotyping"));
    assert_eq!(plan.get("tool_id").and_then(serde_json::Value::as_str), Some("angsd"));
    assert_eq!(
        plan.get("out_dir").and_then(serde_json::Value::as_str),
        Some("target/local-ready/bam.genotyping")
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("sample_id")),
        Some(&serde_json::json!("core-v1-genotyping-panel-sites"))
    );
    assert_eq!(
        plan.get("params").and_then(|params| params.get("producer_contract")),
        Some(&serde_json::json!({
            "bcf": "target/local-ready/bam.genotyping/genotyping.bcf",
            "gl": "target/local-ready/bam.genotyping/genotyping.gl.json",
            "tbi": "target/local-ready/bam.genotyping/genotyping.vcf.gz.tbi",
            "vcf": "target/local-ready/bam.genotyping/genotyping.vcf.gz"
        }))
    );

    let genotyping_bcf = plan["io"]["outputs"]
        .as_array()
        .and_then(|outputs| {
            outputs.iter().find(|artifact| artifact["name"] == serde_json::json!("genotyping_bcf"))
        })
        .unwrap_or_else(|| panic!("genotyping_bcf output missing from CLI materialized plan"));
    assert_eq!(
        genotyping_bcf["path"],
        serde_json::json!("target/local-ready/bam.genotyping/genotyping.bcf")
    );
    assert!(
        plan["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("assets/toy/core-v1/bam/genotyping_panel_sites.sam")
                    && shell.contains("assets/toy/core-v1/bam/genotyping_panel_sites.sam.bai")
                    && shell.contains("assets/toy/core-v1/bam/genotyping_reference_chr1.fasta")
                    && shell.contains("assets/toy/core-v1/vcf/genotyping_candidate_sites.vcf")
                    && shell.contains("assets/toy/core-v1/bam/genotyping_target_regions.txt")
                    && shell.contains("target/local-ready/bam.genotyping/genotyping.bcf")
                    && shell.contains("target/local-ready/bam.genotyping/genotyping.vcf.gz")
            })
        )),
        "CLI materialized genotyping plan must carry the governed BAM, BAI, reference, sites, regions, and variant output paths"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_materialize_stage_bam_kinship_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.kinship",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.kinship"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.kinship/kinship.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.kinship report"),
    )
    .expect("parse bam.kinship report");

    assert_eq!(report.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.kinship"));
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.kinship.local_smoke.report.v1")
    );
    assert_eq!(report.get("case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(report.get("all_cases_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert!(
        report.get("cases").and_then(serde_json::Value::as_array).is_some_and(|cases| {
            cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("core-v1-kinship-insufficient-overlap")
                    && case.get("status").and_then(serde_json::Value::as_str)
                        == Some("insufficient")
                    && case.get("insufficiency_reason").and_then(serde_json::Value::as_str)
                        == Some("insufficient_overlap_snps")
            }) && cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("core-v1-kinship-related-pair")
                    && case.get("status").and_then(serde_json::Value::as_str) == Some("ok")
                    && case.get("pair_count").and_then(serde_json::Value::as_u64) == Some(1)
                    && case
                        .get("pairwise_results")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|pairs| {
                            pairs.iter().any(|pair| {
                                pair.get("sample_a").and_then(serde_json::Value::as_str)
                                    == Some("sample_a")
                                    && pair.get("sample_b").and_then(serde_json::Value::as_str)
                                        == Some("sample_b")
                                    && pair.get("relationship_label").and_then(serde_json::Value::as_str)
                                        == Some("first_degree")
                            })
                        })
            })
        }),
        "bam.kinship local smoke report must preserve the governed insufficient and valid pair cases"
    );
}

#[test]
fn bench_local_materialize_stage_bam_damage_json_writes_governed_smoke_bundle() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "local",
        "materialize-stage",
        "--stage-id",
        "bam.damage",
        "--json",
    ]);

    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.damage"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.damage/damage.json")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("read bam.damage report"),
    )
    .expect("parse bam.damage report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.damage.local_smoke.report.v1")
    );
    assert_eq!(
        report.get("sample_id").and_then(serde_json::Value::as_str),
        Some("adna_damage_non_udg")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("method").and_then(serde_json::Value::as_str), Some("ngsbriggs"));
    assert_eq!(report.get("tools_seen"), Some(&serde_json::json!(["ngsbriggs", "mapdamage2"])));
    assert_eq!(report.get("terminal_c_to_t_5p").and_then(serde_json::Value::as_f64), Some(0.18));
    assert_eq!(report.get("terminal_g_to_a_3p").and_then(serde_json::Value::as_f64), Some(0.11));
    assert_eq!(
        report.get("short_fragment_fraction").and_then(serde_json::Value::as_f64),
        Some(1.0)
    );
    assert_eq!(report.get("damage_signal").and_then(serde_json::Value::as_str), Some("moderate"));
    assert_eq!(
        report.get("strict_profile_upgraded").and_then(serde_json::Value::as_bool),
        Some(false)
    );

    let damage_report = repo_root.join(
        report
            .get("damage_report")
            .and_then(serde_json::Value::as_str)
            .expect("damage report path"),
    );
    let terminal_position_metrics = repo_root.join(
        report
            .get("terminal_position_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("terminal position metrics path"),
    );
    let stage_metrics = repo_root.join(
        report
            .get("stage_metrics")
            .and_then(serde_json::Value::as_str)
            .expect("stage metrics path"),
    );
    assert!(
        damage_report.is_file(),
        "bam.damage smoke bundle must expose the governed damage evidence report"
    );
    assert!(
        terminal_position_metrics.is_file(),
        "bam.damage smoke bundle must expose the governed terminal-position metrics"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.damage smoke bundle must expose the governed stage metrics"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_stage_commands_writes_bash_parseable_51_command_script() {
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
        .args(["bench", "local", "render-stage-commands"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let script_path = repo_root.join("target/local-ready/rendered-stage-commands.sh");
    let manifest_path = repo_root.join("target/local-ready/rendered-stage-commands.json");
    let argv_path = repo_root.join("target/local-ready/rendered-stage-commands.argv.jsonl");
    assert!(script_path.is_file(), "rendered script must exist");
    assert!(manifest_path.is_file(), "rendered JSON manifest must exist");
    assert!(argv_path.is_file(), "rendered argv JSONL companion must exist");

    let syntax = Command::new("bash").arg("-n").arg(&script_path).output().expect("run bash -n");
    assert!(syntax.status.success(), "bash -n failed: {}", String::from_utf8_lossy(&syntax.stderr));

    let script = std::fs::read_to_string(&script_path).expect("read rendered script");
    assert_eq!(
        script.lines().filter(|line| line.starts_with("cargo run -q -p bijux-dna")).count(),
        51
    );

    let argv_jsonl = std::fs::read_to_string(&argv_path).expect("read rendered argv JSONL");
    let argv_rows = argv_jsonl.lines().collect::<Vec<_>>();
    assert_eq!(argv_rows.len(), 51);
    assert!(argv_rows.iter().all(|line| {
        serde_json::from_str::<serde_json::Value>(line).ok().is_some_and(|row| {
            row.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                argv.first().and_then(serde_json::Value::as_str) == Some("cargo")
                    && argv.iter().any(|arg| arg.as_str() == Some("--stage-id"))
            })
        })
    }));
}
