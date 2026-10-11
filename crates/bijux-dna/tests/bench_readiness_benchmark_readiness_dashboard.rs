#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let sandbox =
        support::RepoSandbox::new("benchmark-readiness-dashboard-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");

    sandbox
        .bijux_dna_command()
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
fn bench_readiness_benchmark_readiness_dashboard_tracks_governed_summary_counts() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-benchmark-readiness-dashboard", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.benchmark_readiness_dashboard.v2")
    );
    assert_eq!(payload.get("production_ready").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(payload.get("product_stage_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(payload.get("product_pair_count").and_then(serde_json::Value::as_u64), Some(159));
    assert_eq!(payload.get("active_protocol_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(
        payload.get("reserved_protocol_count").and_then(serde_json::Value::as_u64),
        Some(59)
    );
    assert_eq!(
        payload.get("fixture_covered_pair_count").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(
        payload.get("fixture_excluded_pair_count").and_then(serde_json::Value::as_u64),
        Some(39)
    );
    assert_eq!(
        payload.get("markdown_output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/FASTQ_BAM_FIXTURE_HARNESS_READINESS.md")
    );
    assert_eq!(
        payload.get("json_output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/FASTQ_BAM_FIXTURE_HARNESS_READINESS.json")
    );
    let expected_pair_count = payload
        .get("expected_pair_count")
        .and_then(serde_json::Value::as_u64)
        .expect("expected pair count");
    assert!(expected_pair_count > 0);
    assert_eq!(
        payload.get("ready_pair_count").and_then(serde_json::Value::as_u64),
        Some(expected_pair_count)
    );
    assert_eq!(payload.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("blocker_counts").and_then(serde_json::Value::as_object),
        Some(&serde_json::Map::new())
    );

    let matrix = payload.get("matrix").expect("matrix summary");
    assert_eq!(matrix.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    let tool_count =
        matrix.get("tool_count").and_then(serde_json::Value::as_u64).expect("tool count");
    assert!(tool_count > 0);
    assert_eq!(
        matrix.get("surface_status").and_then(serde_json::Value::as_str),
        Some("fixture_ready")
    );

    let adapters = payload.get("adapters").expect("adapter summary");
    assert_eq!(
        adapters.get("attention_required_pair_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        adapters.get("ready_pair_count").and_then(serde_json::Value::as_u64),
        Some(expected_pair_count)
    );

    let parsers = payload.get("parsers").expect("parser summary");
    let parser_reporting_count = parsers
        .get("benchmark_reporting_pair_count")
        .and_then(serde_json::Value::as_u64)
        .expect("parser reporting count");
    assert_eq!(
        parsers.get("ready_pair_count").and_then(serde_json::Value::as_u64),
        Some(parser_reporting_count)
    );
    assert_eq!(parsers.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(parsers.get("excluded_pair_count").and_then(serde_json::Value::as_u64), Some(2));

    let corpora = payload.get("corpora").expect("corpus summary");
    assert_eq!(corpora.get("corpus_family_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(corpora.get("assigned_stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(corpora.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        corpora
            .get("corpus_family_ids")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "corpus-01",
            "corpus-01-adna-bam",
            "corpus-01-bam",
            "corpus-01-genotyping",
            "corpus-01-kinship",
            "corpus-02",
            "corpus-03",
            "reference-index-assets",
        ])
    );

    let assets = payload.get("assets").expect("asset summary");
    let asset_required_pair_count = assets
        .get("asset_required_pair_count")
        .and_then(serde_json::Value::as_u64)
        .expect("asset required pair count");
    assert!(asset_required_pair_count > 0);
    assert_eq!(
        assets.get("ready_pair_count").and_then(serde_json::Value::as_u64),
        Some(asset_required_pair_count)
    );
    assert_eq!(assets.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(0));

    let reports = payload.get("reports").expect("report summary");
    assert_eq!(reports.get("output_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        reports.get("expected_result_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_pair_count)
    );
    assert_eq!(reports.get("stage_section_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        reports.get("tool_section_count").and_then(serde_json::Value::as_u64),
        Some(tool_count)
    );
    assert_eq!(reports.get("corpus_section_count").and_then(serde_json::Value::as_u64), Some(8));

    let report_outputs = payload
        .get("report_outputs")
        .and_then(serde_json::Value::as_array)
        .expect("report outputs");
    assert_eq!(report_outputs.len(), 5);
    assert!(report_outputs.iter().any(|row| {
        row.get("report_id").and_then(serde_json::Value::as_str) == Some("corpus_centric_report")
            && row.get("item_count").and_then(serde_json::Value::as_u64) == Some(8)
    }));

    let blocked_pairs =
        payload.get("blocked_pairs").and_then(serde_json::Value::as_array).expect("blocked pairs");
    assert!(blocked_pairs.is_empty(), "FASTQ + BAM readiness dashboard must be green");
}
