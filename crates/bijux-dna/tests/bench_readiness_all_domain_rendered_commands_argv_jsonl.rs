#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_rendered_commands_write_governed_argv_jsonl() {
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
        .args(["bench", "readiness", "render-all-domain-commands"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let jsonl_path =
        repo_root.join("benchmarks/readiness/rendered-commands-all-domains.argv.jsonl");
    assert!(jsonl_path.is_file(), "all-domain rendered command argv JSONL must exist");

    let jsonl = std::fs::read_to_string(&jsonl_path).expect("read all-domain command argv JSONL");
    let rows = jsonl.lines().collect::<Vec<_>>();
    assert_eq!(rows.len(), 120);
    assert!(rows.iter().all(|line| {
        serde_json::from_str::<serde_json::Value>(line).ok().is_some_and(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str).is_some()
                && row.get("domain").and_then(serde_json::Value::as_str).is_some()
                && row.get("stage_id").and_then(serde_json::Value::as_str).is_some()
                && row.get("tool_id").and_then(serde_json::Value::as_str).is_some()
                && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("command_steps").and_then(serde_json::Value::as_array).is_some_and(
                    |steps| {
                        !steps.is_empty()
                            && steps.iter().all(|step| {
                                step.get("argv").and_then(serde_json::Value::as_array).is_some_and(
                                    |argv| {
                                        argv.first().and_then(serde_json::Value::as_str).is_some()
                                    },
                                )
                            })
                    },
                )
        })
    }));
    assert!(rows.iter().any(|line| {
        serde_json::from_str::<serde_json::Value>(line).ok().is_some_and(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
                && row.get("command_steps").and_then(serde_json::Value::as_array).is_some_and(
                    |steps| {
                        steps.iter().any(|step| {
                            step.get("argv").and_then(serde_json::Value::as_array).is_some_and(
                                |argv| {
                                    argv.iter()
                                        .any(|token| token.as_str() == Some("bam_downstream"))
                                },
                            )
                        })
                    },
                )
        })
    }));
    assert!(rows.iter().any(|line| {
        serde_json::from_str::<serde_json::Value>(line).ok().is_some_and(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
                && row.get("command_source").and_then(serde_json::Value::as_str)
                    == Some("vcf_bcftools_adapter")
        })
    }));
}
