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
fn bench_readiness_bam_rendered_commands_report_tracks_active_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-bam-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_rendered_commands.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/bam-rendered-commands.sh")
    );
    assert_eq!(
        payload.get("argv_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/bam-rendered-commands.argv.jsonl")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(
        payload
            .get("command_source_counts")
            .and_then(serde_json::Value::as_object)
            .and_then(|counts| counts.get("fastq_bam_command_adapter"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(payload.get("bash_syntax_passed").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 49);
    assert!(rows.iter().all(|row| {
        row.get("benchmark_status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("fastq_bam_command_adapter")
            && row.get("command_steps").and_then(serde_json::Value::as_array).is_some_and(|steps| {
                steps.len() == 1
                    && steps[0].get("step_id").and_then(serde_json::Value::as_str) == Some("invoke")
                    && steps[0]
                        .get("argv")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|argv| !argv.is_empty())
            })
            && row.get("script_commands").and_then(serde_json::Value::as_array).is_some_and(
                |commands| {
                    commands.len() == 1 && commands[0].as_str().is_some_and(|cmd| !cmd.is_empty())
                },
            )
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bwa")
            && row
                .get("command_steps")
                .and_then(serde_json::Value::as_array)
                .and_then(|steps| steps[0].get("argv"))
                .and_then(serde_json::Value::as_array)
                .and_then(|argv| argv.first())
                .and_then(serde_json::Value::as_str)
                == Some("/bin/sh")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("schmutzi")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("reference_fasta+reference_panel")
            && row
                .get("script_commands")
                .and_then(serde_json::Value::as_array)
                .and_then(|commands| commands[0].as_str())
                .is_some_and(|command| command.contains("schmutzi --bam"))
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("king")
            && row
                .get("command_steps")
                .and_then(serde_json::Value::as_array)
                .and_then(|steps| steps[0].get("argv"))
                .and_then(serde_json::Value::as_array)
                .and_then(|argv| argv.first())
                .and_then(serde_json::Value::as_str)
                == Some("cargo")
    }));

    let repo_root = support::repo_root().expect("repo root");
    let argv_jsonl = std::fs::read_to_string(
        repo_root.join("benchmarks/readiness/bam/bam-rendered-commands.argv.jsonl"),
    )
    .expect("read BAM rendered command argv jsonl");
    let argv_rows = argv_jsonl.lines().collect::<Vec<_>>();
    assert_eq!(argv_rows.len(), 49);
    let genotyping = argv_rows
        .iter()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("argv row json"))
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
        })
        .expect("genotyping argv row");
    assert_eq!(
        genotyping.get("command_source").and_then(serde_json::Value::as_str),
        Some("fastq_bam_command_adapter")
    );
    assert!(genotyping.get("command_steps").and_then(serde_json::Value::as_array).is_some_and(
        |steps| {
            steps[0].get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                argv.first().and_then(serde_json::Value::as_str) == Some("cargo")
            })
        }
    ));
}
