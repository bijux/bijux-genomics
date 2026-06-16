#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_apptainer_map_writes_governed_tsv_file() {
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/tools/apptainer-map.tsv");

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
    assert_eq!(rows.len(), 71);
    assert!(rows.iter().any(|row| {
        row == &"adapterremoval\tfastq\tfastq.merge_pairs,fastq.trim_reads,fastq.trim_terminal_damage\tdocker-arm64\tdocker-daemon://bijuxdna/adapterremoval:2.3.3-arm64\tbijuxdna/adapterremoval:2.3.3-arm64\tcontainers/docker/arm64/Dockerfile.adapterremoval\tcontainers/apptainer/shared/adapterremoval.def\t5b618834ce9fc6376c9605c3a69d738236b9be48fdf493c1bc0945568a50808d\t${BIJUX_HPC_ROOT}/.cache\t${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/adapterremoval/5b618834ce9fc6376c9605c3a69d738236b9be48fdf493c1bc0945568a50808d.sif\tapptainer build --force '${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/adapterremoval/5b618834ce9fc6376c9605c3a69d738236b9be48fdf493c1bc0945568a50808d.sif' 'docker-daemon://bijuxdna/adapterremoval:2.3.3-arm64'\tdomain/fastq/tools/adapterremoval.yaml\tconfigs/ci/registry/tool_registry.toml"
    }));
    assert!(rows.iter().any(|row| {
        row == &"angsd\tbam\tbam.genotyping,bam.kinship,bam.sex\tdocker-arm64\tdocker-daemon://bijuxdna/angsd:0.940-arm64\tbijuxdna/angsd:0.940-arm64\tcontainers/docker/arm64/Dockerfile.angsd\tcontainers/apptainer/shared/angsd.def\t55136e06d4ef55e1e64566d3a70fca4bc413cecbc7a1347e9fe80f32f8a7b313\t${BIJUX_HPC_ROOT}/.cache\t${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/angsd/55136e06d4ef55e1e64566d3a70fca4bc413cecbc7a1347e9fe80f32f8a7b313.sif\tapptainer build --force '${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/angsd/55136e06d4ef55e1e64566d3a70fca4bc413cecbc7a1347e9fe80f32f8a7b313.sif' 'docker-daemon://bijuxdna/angsd:0.940-arm64'\tdomain/bam/tools/angsd.yaml\tconfigs/ci/registry/tool_registry.toml,configs/ci/registry/tool_registry_vcf.toml"
    }));
    assert!(rows.iter().any(|row| {
        row == &"shapeit5\tvcf\tvcf.phasing\tdocker-arm64\tdocker-daemon://bijuxdna/shapeit5:5.1.1-arm64\tbijuxdna/shapeit5:5.1.1-arm64\tcontainers/docker/arm64/Dockerfile.shapeit5\tcontainers/apptainer/shared/shapeit5.def\t6c2e2eb0becbcb11bbc83523fcf826adf79cf63b65993a12c660e7d19e64ff61\t${BIJUX_HPC_ROOT}/.cache\t${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/shapeit5/6c2e2eb0becbcb11bbc83523fcf826adf79cf63b65993a12c660e7d19e64ff61.sif\tapptainer build --force '${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/shapeit5/6c2e2eb0becbcb11bbc83523fcf826adf79cf63b65993a12c660e7d19e64ff61.sif' 'docker-daemon://bijuxdna/shapeit5:5.1.1-arm64'\tdomain/vcf/tools/shapeit5.yaml\tconfigs/ci/registry/tool_registry_vcf.toml"
    }));
}
