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
fn bench_readiness_vcf_rendered_commands_report_tracks_canonical_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_rendered_commands.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/vcf-rendered-commands.sh")
    );
    assert_eq!(
        payload.get("argv_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/vcf-rendered-commands.argv.jsonl")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(18));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 18);
    assert!(rows.iter().all(|row| {
        row.get("benchmark_status").and_then(serde_json::Value::as_str) == Some("benchmark_ready")
            && row.get("readiness_kind").and_then(serde_json::Value::as_str)
                == Some("benchmark_ready")
            && row
                .get("command_steps")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|steps| !steps.is_empty())
            && row
                .get("script_commands")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|commands| !commands.is_empty())
    }));

    let vcf_call = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
        })
        .expect("vcf.call row");
    assert_eq!(
        vcf_call.get("command_steps").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(3)
    );
    assert!(
        vcf_call.get("script_commands").and_then(serde_json::Value::as_array).is_some_and(
            |commands| {
                commands.iter().any(|item| {
                    item.as_str().is_some_and(|command| {
                        command.contains("bcftools mpileup")
                            && command.contains(" | bcftools call ")
                    })
                })
            }
        ),
        "vcf.call row must preserve the mpileup->call pipeline"
    );

    let vcf_stats = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
        })
        .expect("vcf.stats row");
    assert_eq!(
        vcf_stats.get("command_steps").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(1)
    );

    let vcf_qc = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
        })
        .expect("vcf.qc row");
    assert_eq!(
        vcf_qc.get("command_steps").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(2)
    );
    assert!(
        vcf_qc.get("script_commands").and_then(serde_json::Value::as_array).is_some_and(
            |commands| commands.iter().any(|item| {
                item.as_str().is_some_and(|command| {
                    command.contains("bcftools query") && command.contains("raw.genotypes.tsv")
                })
            })
        ),
        "vcf.qc row must preserve the bcftools genotype extraction step"
    );

    let vcf_postprocess = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
        })
        .expect("vcf.postprocess row");
    assert_eq!(
        vcf_postprocess.get("command_steps").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(2)
    );
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation_metrics")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("command_steps").and_then(serde_json::Value::as_array).map(Vec::len)
                == Some(1)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eigensoft")
            && row.get("command_steps").and_then(serde_json::Value::as_array).map(Vec::len)
                == Some(4)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("vcf.prepare_reference_panel")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
    }));
}
