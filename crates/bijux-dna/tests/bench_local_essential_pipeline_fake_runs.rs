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
fn bench_local_fake_run_essential_pipelines_json_reports_governed_pipeline_slice() {
    let payload = run_cli_json(&["bench", "local", "fake-run-essential-pipelines", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_essential_pipeline_fake_runs.v1")
    );
    assert_eq!(
        payload.get("fake_run_root").and_then(serde_json::Value::as_str),
        Some("target/local-fake-runs/pipelines/essential")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(93));
    assert!(payload
        .get("created_output_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 93));

    let pipelines =
        payload.get("pipelines").and_then(serde_json::Value::as_array).expect("pipelines array");
    assert_eq!(pipelines.len(), 10);

    let core = pipelines
        .iter()
        .find(|pipeline| {
            pipeline.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("core-germline-fastq-bam-vcf")
        })
        .expect("core germline pipeline row");
    assert_eq!(core.get("node_count").and_then(serde_json::Value::as_u64), Some(12));
    assert!(core
        .get("topological_order")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|order| order.first().and_then(serde_json::Value::as_str)
            == Some("fastq.validate_reads")));

    let bam_align = core
        .get("nodes")
        .and_then(serde_json::Value::as_array)
        .expect("core nodes")
        .iter()
        .find(|node| node.get("node_id").and_then(serde_json::Value::as_str) == Some("bam.align"))
        .expect("bam.align node");
    assert_eq!(bam_align.get("tool_id").and_then(serde_json::Value::as_str), Some("bowtie2"));
    assert_eq!(
        bam_align.get("command_source").and_then(serde_json::Value::as_str),
        Some("bam_governed_stage_command")
    );
    assert_eq!(
        bam_align.get("created_output_count").and_then(serde_json::Value::as_u64),
        bam_align.get("declared_output_count").and_then(serde_json::Value::as_u64)
    );
    assert!(bam_align.get("stage_result_path").and_then(serde_json::Value::as_str).is_some_and(
        |path| path.ends_with("/core-germline-fastq-bam-vcf/bam.align/stage-result.json")
    ));

    let report_qc = pipelines
        .iter()
        .find(|pipeline| {
            pipeline.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("edna-taxonomy-no-vcf")
        })
        .and_then(|pipeline| pipeline.get("nodes").and_then(serde_json::Value::as_array))
        .expect("edna nodes")
        .iter()
        .find(|node| {
            node.get("node_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        })
        .expect("fastq.report_qc node");
    assert_eq!(report_qc.get("tool_id").and_then(serde_json::Value::as_str), Some("bijux-dna"));
    assert_eq!(
        report_qc.get("command_source").and_then(serde_json::Value::as_str),
        Some("local_stage_materialization")
    );
}
