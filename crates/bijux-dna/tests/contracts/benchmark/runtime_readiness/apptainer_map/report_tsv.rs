#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

#[test]
fn bench_readiness_apptainer_map_writes_governed_tsv_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-apptainer-map"])
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
    assert_eq!(rendered_path.trim(), "artifacts/benchmark-smoke/readiness/tools/apptainer-map.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read apptainer map tsv");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tdomains\tactive_stage_ids\tdocker_runtime\timage_uri\tlocal_image_name\tdockerfile\tapptainer_def\tapptainer_cache_key\tcache_root\texpected_sif_path\tconversion_command\truntime_probe_paths\tregistry_paths"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|row| {
        !row.starts_with("angsd\t")
            && !row.starts_with("fastuniq\t")
            && !row.starts_with("impute5\t")
            && !row.starts_with("ngsbriggs\t")
    }));
    assert!(rows.iter().any(|row| {
        row == &"adapterremoval\tfastq\tfastq.merge_pairs,fastq.trim_reads,fastq.trim_terminal_damage\tdocker-arm64\tdocker-daemon://bijuxdna/adapterremoval:2.3.4-arm64\tbijuxdna/adapterremoval:2.3.4-arm64\tcontainers/docker/Dockerfile.arm64.adapterremoval\tcontainers/apptainer/adapterremoval.def\t47cc1f41af7de6e9b8057a1ff80748e96df1a1e653bf39d8f6b8321a6f050c20\t${BIJUX_HPC_ROOT}/.cache\t${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/adapterremoval/47cc1f41af7de6e9b8057a1ff80748e96df1a1e653bf39d8f6b8321a6f050c20.sif\tapptainer build --force '${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/adapterremoval/47cc1f41af7de6e9b8057a1ff80748e96df1a1e653bf39d8f6b8321a6f050c20.sif' 'docker-daemon://bijuxdna/adapterremoval:2.3.4-arm64'\tdomain/fastq/tools/adapterremoval.yaml\tconfigs/ci/registry/tool_registry.toml"
    }));
    assert!(rows.iter().all(|row| !row.starts_with("shapeit5\t")));
}
