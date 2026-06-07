#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_render_all_domain_slurm_submit_manifest_writes_governed_manifest_file() {
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
        .args(["bench", "local", "render-all-domain-slurm-submit-manifest"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "runs/bench/slurm-dry-run/all-domains/submit-manifest.json");

    let manifest_path = repo_root.join(rendered_path.trim());
    assert!(manifest_path.is_file(), "all-domain submit manifest must exist");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read manifest"))
            .expect("parse manifest");
    assert_eq!(manifest.get("job_count").and_then(serde_json::Value::as_u64), Some(213));

    let jobs = manifest.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    let benchmark_job = jobs
        .iter()
        .find(|job| {
            job.get("job_id_local").and_then(serde_json::Value::as_str)
                == Some("benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
        })
        .expect("benchmark job");
    assert_eq!(
        benchmark_job
            .get("dependencies")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(0)
    );
    assert_eq!(
        benchmark_job.get("stdout").and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/stdout.log"
        )
    );
    assert!(benchmark_job.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| {
            outputs.iter().any(|value| {
                value.as_str().is_some_and(|path| {
                    path.contains("/declared-outputs/")
                        && path.starts_with("runs/bench/slurm-dry-run/")
                })
            }) && outputs
                .iter()
                .any(|value| value.as_str().is_some_and(|path| path.ends_with("stage-result.json")))
        }
    ));

    let pipeline_job = jobs
        .iter()
        .find(|job| {
            job.get("job_id_local").and_then(serde_json::Value::as_str)
                == Some("pipeline:relatedness-segments-vcf:vcf.demography")
        })
        .expect("pipeline job");
    assert_eq!(
        pipeline_job
            .get("dependencies")
            .and_then(serde_json::Value::as_array)
            .map(|rows| { rows.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>() }),
        Some(vec!["pipeline:relatedness-segments-vcf:vcf.ibd"])
    );
    assert_eq!(
        pipeline_job.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        pipeline_job.get("stdout").and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/relatedness-segments-vcf/vcf.demography/ibdne/vcf_production_regression/sample-set/stdout.log"
        )
    );
    assert!(
        pipeline_job
            .get("outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs.iter().any(|value| {
                value.as_str().is_some_and(|path| path.contains("/runs/all-domain-benchmark-dry-run/vcf/relatedness-segments-vcf/vcf.demography/"))
            }))
    );
}
