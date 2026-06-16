#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_apptainer_map_reports_governed_docker_to_sif_mappings() {
    let payload = run_cli_json(&["bench", "readiness", "render-apptainer-map", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.apptainer_map.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/apptainer-map.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(
        payload.get("docker_runtime").and_then(serde_json::Value::as_str),
        Some("docker-arm64")
    );
    assert_eq!(
        payload.get("cache_root").and_then(serde_json::Value::as_str),
        Some("${BIJUX_HPC_ROOT}/.cache")
    );

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(42));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(6));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 71);

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("adapterremoval")
            && row.get("image_uri").and_then(serde_json::Value::as_str)
                == Some("docker-daemon://bijuxdna/adapterremoval:2.3.3-arm64")
            && row.get("expected_sif_path").and_then(serde_json::Value::as_str).is_some_and(
                |path| path.ends_with(
                    "/adapterremoval/5b618834ce9fc6376c9605c3a69d738236b9be48fdf493c1bc0945568a50808d.sif"
                )
            )
            && row.get("registry_paths").and_then(serde_json::Value::as_array).is_some_and(
                |paths| paths.len() == 1
                    && paths.first().and_then(serde_json::Value::as_str)
                        == Some("configs/ci/registry/tool_registry.toml")
            )
    }));

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
            && row.get("image_uri").and_then(serde_json::Value::as_str)
                == Some("docker-daemon://bijuxdna/angsd:0.940-arm64")
            && row.get("registry_paths").and_then(serde_json::Value::as_array).is_some_and(
                |paths| {
                    paths
                        .iter()
                        .any(|path| path.as_str() == Some("configs/ci/registry/tool_registry.toml"))
                        && paths.iter().any(|path| {
                            path.as_str() == Some("configs/ci/registry/tool_registry_vcf.toml")
                        })
                },
            )
    }));

    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("image_uri").and_then(serde_json::Value::as_str)
                == Some("docker-daemon://bijuxdna/shapeit5:5.1.1-arm64")
            && row.get("conversion_command").and_then(serde_json::Value::as_str).is_some_and(
                |command| {
                    command.contains("apptainer build --force")
                        && command.contains("docker-daemon://bijuxdna/shapeit5:5.1.1-arm64")
                },
            )
            && row.get("runtime_probe_paths").and_then(serde_json::Value::as_array).is_some_and(
                |paths| {
                    paths.len() == 1
                        && paths.first().and_then(serde_json::Value::as_str)
                            == Some("domain/vcf/tools/shapeit5.yaml")
                },
            )
    }));
}
