#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_essential_pipeline_rendered_commands_report_tracks_governed_nodes() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-essential-pipeline-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipeline_rendered_commands.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/essential-pipelines-rendered-commands.sh")
    );
    assert_eq!(
        payload.get("argv_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/essential-pipelines-rendered-commands.argv.jsonl")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(payload.get("rendered_row_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(
        payload.get("structured_skip_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(29)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(31)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("vcf"))
            .and_then(serde_json::Value::as_u64),
        Some(33)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 93);
    assert!(rows.iter().all(|row| {
        row.get("render_status").and_then(serde_json::Value::as_str) == Some("rendered")
            && row.get("skip").is_some_and(serde_json::Value::is_null)
            && row
                .get("command_steps")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|steps| !steps.is_empty())
            && row
                .get("script_commands")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|commands| !commands.is_empty())
    }));

    let phasing_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("reference-panel-imputation")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
        })
        .expect("reference-panel phasing row");
    assert_eq!(phasing_row.get("tool_id").and_then(serde_json::Value::as_str), Some("shapeit5"));
    assert_eq!(
        phasing_row.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_default_tool_adapter")
    );
    assert!(
        phasing_row.get("script_commands").and_then(serde_json::Value::as_array).is_some_and(
            |commands| commands.iter().any(|item| {
                item.as_str().is_some_and(|command| command.contains("shapeit5 phase_common"))
            })
        ),
        "reference-panel phasing row must keep the shapeit5 command"
    );

    let report_qc_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("edna-taxonomy-no-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        })
        .expect("eDNA report_qc row");
    assert_eq!(report_qc_row.get("tool_id").and_then(serde_json::Value::as_str), Some("bijux-dna"));
    assert_eq!(
        report_qc_row.get("command_source").and_then(serde_json::Value::as_str),
        Some("local_stage_materialization")
    );
    assert!(
        report_qc_row.get("script_commands").and_then(serde_json::Value::as_array).is_some_and(
            |commands| commands.iter().any(|item| {
                item.as_str().is_some_and(|command| {
                    command.contains("bench local materialize-stage --stage-id fastq.report_qc")
                })
            })
        ),
        "fastq.report_qc row must render the owned local-stage materialization command"
    );

    let bam_genotyping_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("bam-genotyping-to-vcf-downstream")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
        })
        .expect("bam.genotyping row");
    assert_eq!(
        bam_genotyping_row.get("tool_id").and_then(serde_json::Value::as_str),
        Some("bijux-dna")
    );
    assert_eq!(
        bam_genotyping_row.get("command_source").and_then(serde_json::Value::as_str),
        Some("local_stage_materialization")
    );
    assert!(
        bam_genotyping_row
            .get("script_commands")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|commands| commands.iter().any(|item| {
                item.as_str().is_some_and(|command| {
                    command.contains("bench local materialize-stage --stage-id bam.genotyping")
                })
            })),
        "bam.genotyping row must render the owned local-stage materialization command"
    );
}
