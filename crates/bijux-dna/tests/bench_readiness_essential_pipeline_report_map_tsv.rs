#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_essential_pipeline_report_map_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-essential-pipeline-report-map"])
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
    assert_eq!(rendered_path.trim(), "target/bench-readiness/essential-pipeline-report-map.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read essential pipeline report map tsv");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some("pipeline_id\tstage_id\ttool_id\toutput_metric\treport_section\tfailure_column")
    );
    assert!(lines.any(|line| {
        line
            == "core-germline-fastq-bam-vcf\tvcf.call\tbcftools\tcalled_vcf\tvariant_calling\tfailure_reason"
    }));
    assert!(lines.any(|line| {
        line
            == "edna-taxonomy-no-vcf\tfastq.screen_taxonomy\tkraken2\ttaxonomy_summary\tcontamination_screening\tfailure_reason"
    }));
    assert!(lines.any(|line| {
        line
            == "relatedness-segments-vcf\tvcf.demography\tibdne\tdemography_report\tdemography\tfailure_reason"
    }));
}
