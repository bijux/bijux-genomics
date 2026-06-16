#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_executable_resolution_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-executable-resolution"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/tools/executable-resolution.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read executable resolution tsv");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tdomains\tactive_stage_ids\tinstall_kind\tresolution_kind\tresolution_target\tcommand_entrypoint\truntime_probe_paths\tunavailable_reason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 71);
    assert!(rows.iter().any(|row| {
        row == &"bijux_dna\tfastq\tfastq.detect_duplicates_premerge,fastq.estimate_library_complexity_prealign\tworkspace_binary\thost_binary\tbijux-dna\tbijux-dna\tdomain/fastq/tools/bijux_dna.yaml\t"
    }));
    assert!(rows.iter().any(|row| {
        row == &"samtools\tbam\tbam.coverage,bam.duplication_metrics,bam.endogenous_content,bam.filter,bam.length_filter,bam.mapping_summary,bam.mapq_filter,bam.markdup,bam.qc_pre,bam.validate\tcontainer\tdocker_image\tbijuxdna/samtools:1.21\tsamtools\tdomain/bam/tools/samtools.yaml\t"
    }));
    assert!(rows.iter().any(|row| {
        row == &"beagle\tvcf\tvcf.imputation_metrics,vcf.impute\tcontainer\tapptainer_image\tcontainers/apptainer/lunarc/beagle.def@sha256:220b8f1687f32f6f04cb4e85b0d6ab4ecd2e98f6f5147064c4c2420ddfdd5b3f\tbeagle\tdomain/vcf/tools/beagle.yaml\t"
    }));
    assert!(rows.iter().any(|row| {
        row == &"shapeit5\tvcf\tvcf.phasing\tcontainer\tunavailable_with_reason\t\tshapeit5\tdomain/vcf/tools/shapeit5.yaml\truntime probe declares an external container source without a governed local image"
    }));
}
