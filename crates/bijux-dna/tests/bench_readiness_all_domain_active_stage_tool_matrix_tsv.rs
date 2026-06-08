#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_active_stage_tool_matrix_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-active-stage-tool-matrix"])
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
    assert_eq!(
        rendered_path.trim(),
        "benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain active stage-tool matrix");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tstatus"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 125);
    assert!(rows.iter().any(|row| {
        row == &"bam\tbam.contamination\tschmutzi\tcorpus-01-adna-bam-mini\treference_fasta+reference_panel\tbam.adapter.contamination\tbam.parser.contamination\tbam_contamination_normalized_v1\tbenchmark_ready"
    }));
    assert!(rows.iter().any(|row| {
        row == &"fastq\tfastq.trim_reads\ttrimmomatic\tcorpus-01-mini\tcorpus_only\tfastq.adapter.trim_reads\tfastq.parser.trim_reads\tfastq_trim_reads_v2\tbenchmark_ready"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf\tvcf.stats\tbcftools\tvcf_production_regression\tvcf_cohort\tvcf.adapter.quality_control\tvcf.parser.stats_report\tbijux.schemas.bench.vcf-normalized-metrics.stats.v1\tbenchmark_ready"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tvcf.adapter.transform\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.postprocess.v1\tbenchmark_ready"
    }));
    assert!(
        rows.iter().all(|row| row.ends_with("\tbenchmark_ready")),
        "active scope TSV must keep only benchmark-ready rows"
    );
    assert!(
        rows.iter().all(|row| {
            !row.contains("\tfastq.index_reference\t")
                && !row.contains("\tfastq.profile_overrepresented_sequences\t")
                && !row.contains("\tfastq.report_qc\t")
        }),
        "active scope TSV must exclude not-benchmark-ready-only stages"
    );
}
