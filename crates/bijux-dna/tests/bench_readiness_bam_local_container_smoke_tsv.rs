#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_local_container_smoke_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-bam-local-container-smoke"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/bam/bam-local-container-smoke.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM local-container smoke TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\tregistered_binary\ttool_status\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tsmoke_path_kind\tsmoke_runtime\tsmoke_tool_id\tsmoke_command\tsmoke_support_path\tsmoke_minimal_cmd\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 49);
    assert!(rows.iter().any(|row| {
        row == &"bam.validate\tsamtools\tsamtools\tproduction\tbenchmark_ready\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\thost_stage_smoke\thost\tsamtools\tbijux-dna bench local run-bam-stage-smoke --stage-id bam.validate\tcrates/bijux-dna/src/commands/benchmark/local_bam_stage_smoke.rs\t\tbinding `bam.validate` / `samtools` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam.coverage\tsamtools\tsamtools\tproduction\tbenchmark_ready\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\thost_stage_smoke\thost\tsamtools\tbijux-dna bench local run-bam-stage-smoke --stage-id bam.coverage\tcrates/bijux-dna/src/commands/benchmark/local_bam_stage_smoke.rs\t\tbinding `bam.coverage` / `samtools` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam.coverage\tmosdepth\tmosdepth\tproduction\tbenchmark_ready\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tdocker_container_smoke\tdocker-arm64\tmosdepth\tbijux-dna env smoke docker-arm64 mosdepth\tcontainers/docker/arm64/Dockerfile.mosdepth\t\tbinding `bam.coverage` / `mosdepth` does not match the governed BAM local-smoke contract tool `samtools`, so the governed container smoke wrapper is the available local exercise path for `bam.coverage` / `mosdepth`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam.align\tbwa\tbwa\tproduction\tbenchmark_ready\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-mini\tdocker_container_smoke\tdocker-arm64\tbwa\tbijux-dna env smoke docker-arm64 bwa\tcontainers/docker/arm64/Dockerfile.bwa\t\tstage `bam.align` keeps governed local-ready plan coverage but no BAM tiny-fixture smoke wrapper, so the governed container smoke wrapper is the available local exercise path for `bam.align` / `bwa`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam.contamination\tverifybamid2\tverifybamid2\tproduction\tbenchmark_ready\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-adna-bam-mini\tdocker_container_smoke\tdocker-arm64\tverifybamid2\tbijux-dna env smoke docker-arm64 verifybamid2\tcontainers/docker/arm64/Dockerfile.verifybamid2\t\tstage `bam.contamination` keeps governed local-ready plan coverage but no BAM tiny-fixture smoke wrapper, so the governed container smoke wrapper is the available local exercise path for `bam.contamination` / `verifybamid2`"
    }));
}
