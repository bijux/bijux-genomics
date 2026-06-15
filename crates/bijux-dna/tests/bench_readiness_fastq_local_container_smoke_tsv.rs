#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_local_container_smoke_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-fastq-local-container-smoke"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/fastq/fastq-local-container-smoke.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ local-container smoke TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\tregistered_binary\ttool_status\tbenchmark_status\tsupport_status\tcorpus_status\tsmoke_path_kind\tsmoke_runtime\tsmoke_tool_id\tsmoke_command\tsmoke_support_path\tsmoke_minimal_cmd\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 69);
    assert!(rows.iter().any(|row| {
        row == &"fastq.detect_duplicates_premerge\tbijux_dna\tbijux-dna\tproduction\tbenchmark_ready\tgoverned_execution\tfixture:corpus-01-mini\tdocker_container_smoke\tdocker-arm64\tbijux-dna\tbijux-dna env smoke docker-arm64 bijux-dna\tcontainers/docker/arm64/Dockerfile.bijux_dna\t\tbinding `fastq.detect_duplicates_premerge` / `bijux_dna` matches the governed FASTQ execution default tool, but no exact tiny-fixture stage smoke wrapper is checked in, so the governed container smoke wrapper is the available local exercise path for `fastq.detect_duplicates_premerge` / `bijux_dna`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"fastq.normalize_primers\tcutadapt\tcutadapt\tproduction\tbenchmark_ready\tgoverned_benchmark_cohort\tfixture:corpus-03-amplicon-mini\tdocker_container_smoke\tdocker-arm64\tcutadapt\tbijux-dna env smoke docker-arm64 cutadapt\tcontainers/docker/arm64/Dockerfile.cutadapt\t\tbinding `fastq.normalize_primers` / `cutadapt` matches the governed FASTQ execution default tool, but no exact tiny-fixture stage smoke wrapper is checked in, so the governed container smoke wrapper is the available local exercise path for `fastq.normalize_primers` / `cutadapt`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"fastq.infer_asvs\tdada2\tdada2\tproduction\tbenchmark_ready\tgoverned_execution\tfixture:corpus-03-amplicon-mini\tdocker_container_smoke\tdocker-arm64\tdada2\tbijux-dna env smoke docker-arm64 dada2\tcontainers/docker/arm64/Dockerfile.dada2\t\tbinding `fastq.infer_asvs` / `dada2` matches the governed FASTQ execution default tool, but no exact tiny-fixture stage smoke wrapper is checked in, so the governed container smoke wrapper is the available local exercise path for `fastq.infer_asvs` / `dada2`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"fastq.normalize_abundance\tseqkit\tseqkit\tproduction\tbenchmark_ready\tgoverned_benchmark_cohort\tfixture:corpus-03-amplicon-mini\tdocker_container_smoke\tdocker-arm64\tseqkit\tbijux-dna env smoke docker-arm64 seqkit\tcontainers/docker/arm64/Dockerfile.seqkit\t\tbinding `fastq.normalize_abundance` / `seqkit` matches the governed FASTQ execution default tool, but no exact tiny-fixture stage smoke wrapper is checked in, so the governed container smoke wrapper is the available local exercise path for `fastq.normalize_abundance` / `seqkit`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"fastq.validate_reads\tfastq_scan\tfastq_scan\tproduction\tbenchmark_ready\tobserver_specialized_benchmark\tfixture:corpus-01-mini\tdocker_container_smoke\tdocker-arm64\tfastq_scan\tbijux-dna env smoke docker-arm64 fastq_scan\tcontainers/docker/arm64/Dockerfile.fastq_scan\t\tretained tool `fastq_scan` has no exact tiny-fixture stage smoke wrapper, so the governed container smoke wrapper is the available local exercise path for `fastq.validate_reads` / `fastq_scan`"
    }));
}
