#![allow(clippy::expect_used)]

use std::path::PathBuf;
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
        Some("bijux.bench.local_stage_commands.v2")
    );
    assert_eq!(
        payload.get("script_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/rendered-stage-commands.sh")
    );
    assert_eq!(
        payload.get("manifest_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/rendered-stage-commands.json")
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
                        == Some("core-v1-duplicate-contigs")
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
        "core-v1-partial-mapping\t3\t2\t1\t0.666667\t0\t0\tchr1\ttrue\ttarget/local-smoke/bam.mapping_summary/core-v1-partial-mapping/samtools/mapping.summary.json\ttarget/local-smoke/bam.mapping_summary/core-v1-partial-mapping/samtools/flagstat.txt\ttarget/local-smoke/bam.mapping_summary/core-v1-partial-mapping/samtools/idxstats.txt\ttarget/local-smoke/bam.mapping_summary/core-v1-partial-mapping/samtools/samtools_stats.txt\ttarget/local-smoke/bam.mapping_summary/core-v1-partial-mapping/samtools/stage.metrics.json"
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
        Some("core-v1-general-filter")
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
        Some("core-v1-mapq-threshold")
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
        Some("core-v1-length-threshold")
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
        Some("core-v1-markdup-cluster")
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
        Some("core-v1-duplicate-observation")
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

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.complexity")
    );
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
        Some("core-v1-complexity-insufficient")
    );
    assert_eq!(report.get("expectation_matched").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("observed_total_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("observed_unique_reads").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(report.get("estimated_unique_reads"), Some(&serde_json::Value::Null));
    assert_eq!(report.get("estimated_library_size"), Some(&serde_json::Value::Null));
    assert_eq!(report.get("saturation_estimate"), Some(&serde_json::Value::Null));
    assert_eq!(
        report.get("insufficient_data_reason").and_then(serde_json::Value::as_str),
        Some("insufficient_observed_unique_reads_for_complexity_extrapolation")
    );

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

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.coverage")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.coverage/coverage.tsv")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let coverage_tsv = std::fs::read_to_string(&artifact_path).expect("read bam.coverage summary");
    assert!(coverage_tsv.contains("sample_id\tregion_id\tcontig"));
    assert!(coverage_tsv.contains("core-v1-target-windows\tchr1_window\tchr1"));
    assert!(coverage_tsv.contains("core-v1-target-windows\tchr2_window\tchr2"));
    assert!(coverage_tsv.contains("low_pass\ttrue\ttrue"));

    let case_summary = repo_root.join(
        "target/local-smoke/bam.coverage/core-v1-target-windows/samtools/coverage.summary.json",
    );
    let stage_metrics = repo_root
        .join("target/local-smoke/bam.coverage/core-v1-target-windows/samtools/stage.metrics.json");
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
        Some("core-v1-paired-triplet")
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

    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.gc_bias")
    );
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/bam.gc_bias/gc_bias.tsv")
    );

    let artifact_path = repo_root.join(
        payload.get("artifact_path").and_then(serde_json::Value::as_str).expect("artifact path"),
    );
    let gc_bias_tsv = std::fs::read_to_string(&artifact_path).expect("read bam.gc_bias summary");
    assert!(gc_bias_tsv.contains("sample_id\tgc_bin\tnormalized_coverage\twindows\tread_starts"));
    assert!(gc_bias_tsv.contains("core-v1-gc-window-ladder\t0\t0.750000\t1\t1"));
    assert!(gc_bias_tsv.contains("core-v1-gc-window-ladder\t50\t1.500000\t1\t2"));
    assert!(gc_bias_tsv.contains("core-v1-gc-window-ladder\t100\t0.750000\t1\t1"));

    let gc_bias_summary = repo_root
        .join("target/local-smoke/bam.gc_bias/core-v1-gc-window-ladder/picard/gc_bias.summary.json");
    let stage_metrics =
        repo_root.join("target/local-smoke/bam.gc_bias/core-v1-gc-window-ladder/picard/stage.metrics.json");
    assert!(
        gc_bias_summary.is_file(),
        "bam.gc_bias smoke bundle must expose the governed gc-bias summary json"
    );
    assert!(
        stage_metrics.is_file(),
        "bam.gc_bias smoke bundle must expose the governed stage metrics"
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
    assert!(script_path.is_file(), "rendered script must exist");
    assert!(manifest_path.is_file(), "rendered JSON manifest must exist");

    let syntax = Command::new("bash").arg("-n").arg(&script_path).output().expect("run bash -n");
    assert!(syntax.status.success(), "bash -n failed: {}", String::from_utf8_lossy(&syntax.stderr));

    let script = std::fs::read_to_string(&script_path).expect("read rendered script");
    assert_eq!(
        script.lines().filter(|line| line.starts_with("cargo run -q -p bijux-dna")).count(),
        51
    );
}
