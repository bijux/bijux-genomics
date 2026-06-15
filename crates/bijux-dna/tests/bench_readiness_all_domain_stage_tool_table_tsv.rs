#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_stage_tool_table_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-stage-tool-table"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domain-stage-tool-table.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain stage tool table");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tbenchmark_status"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 146);
    assert!(rows.iter().any(|row| {
        row == &"fastq\tfastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tfastq.adapter.screen_taxonomy\tfastq.parser.screen_taxonomy\tbenchmark_ready"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam\tbam.contamination\tschmutzi\tcorpus-01-adna-bam-mini\treference_fasta+reference_panel\tbam.adapter.contamination\tbam.parser.contamination\tbenchmark_ready"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf\tvcf.call\tbcftools\tvcf_production_regression\tbam_bundle\tvcf.adapter.calling\tvcf.parser.call_summary\tbenchmark_ready"
    }));
}
