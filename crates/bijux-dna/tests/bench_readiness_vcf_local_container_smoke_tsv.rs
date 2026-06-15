#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_local_container_smoke_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-vcf-local-container-smoke"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf/vcf-local-container-smoke.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF local-container smoke TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\tregistered_binary\ttool_status\tstage_support_status\tscope_state\tscope_detail\tsmoke_path_kind\tsmoke_runtime\tsmoke_tool_id\tsmoke_command\tsmoke_support_path\tsmoke_minimal_cmd\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 42);
    assert!(rows.iter().any(|row| {
        row == &"vcf.call\tbcftools\tbcftools\tproduction\tsupported\tactive\tactive\thost_stage_smoke\thost\tbcftools\tbijux-dna bench local run-vcf-call-smoke --tool-id bcftools\tcrates/bijux-dna/src/commands/benchmark/local_vcf_call_smoke.rs\t\tbinding `vcf.call` / `bcftools` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.ibd\tgermline\tgermline\texperimental,planned\tplanned\tremoved_from_scope\tlifecycle_not_active\thost_stage_smoke\thost\tgermline\tbijux-dna bench local run-vcf-ibd-smoke --tool-id germline\tcrates/bijux-dna/src/commands/benchmark/local_vcf_ibd_smoke.rs\t\tbinding `vcf.ibd` / `germline` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.imputation_metrics\tbeagle\tbeagle\tproduction\tsupported\tactive\tactive\thost_stage_smoke\thost\tbeagle\tbijux-dna bench local run-vcf-imputation-metrics-smoke --tool-id beagle\tcrates/bijux-dna/src/commands/benchmark/local_vcf_imputation_metrics_smoke.rs\t\tbinding `vcf.imputation_metrics` / `beagle` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.impute\tbeagle\tbeagle\tproduction\tsupported\tactive\tactive\thost_stage_smoke\thost\tbeagle\tbijux-dna bench local run-vcf-impute-smoke --tool-id beagle\tcrates/bijux-dna/src/commands/benchmark/local_vcf_impute_smoke.rs\t\tbinding `vcf.impute` / `beagle` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.postprocess\tbcftools\tbcftools\tproduction\tsupported\tactive\tactive\tdocker_container_smoke\tdocker-arm64\tbcftools\tbijux-dna env smoke docker-arm64 bcftools\tcontainers/docker/arm64/Dockerfile.bcftools\t\tbinding `vcf.postprocess` / `bcftools` matches the governed VCF stage-matrix default tool, but no exact tiny-fixture stage smoke wrapper is checked in, so the governed container smoke wrapper is the available local exercise path for `vcf.postprocess` / `bcftools`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.phasing\tshapeit\tshapeit\tplanned\tsupported\tremoved_from_scope\tbenchmark_not_ready\tapptainer_container_smoke\tapptainer\tshapeit\tbijux-dna env smoke apptainer shapeit\tcontainers/apptainer/shared/shapeit.def\tshapeit --help\tretained tool `shapeit` has no exact tiny-fixture stage smoke wrapper, so the governed container smoke wrapper is the available local exercise path for `vcf.phasing` / `shapeit`; no-run-possible: planned wrapper surface exposes help/version only until packaging and phasing fixtures are governed."
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.ibd\tibdseq\tibdseq\tplanned\tplanned\tremoved_from_scope\tlifecycle_not_active\tdocker_container_smoke\tdocker-arm64\tibdseq\tbijux-dna env smoke docker-arm64 ibdseq\tcontainers/docker/arm64/Dockerfile.ibdseq\tibdseq --help\tretained tool `ibdseq` has no exact tiny-fixture stage smoke wrapper, so the governed container smoke wrapper is the available local exercise path for `vcf.ibd` / `ibdseq`; no-run-possible: planned wrapper surface exposes help/version only until packaging and IBD fixtures are governed."
    }));
}
