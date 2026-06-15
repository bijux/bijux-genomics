#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_active_stage_tool_matrix_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-fastq-active-stage-tool-matrix"])
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
        "benchmarks/readiness/fastq/fastq-active-stage-tool-matrix.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ active-stage-tool matrix TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tscope_proof_path\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 69);
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "fastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tfastq.adapter.screen_taxonomy\tfastq.parser.screen_taxonomy\tfastq_screen_taxonomy_v1\tbenchmark_ready\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-02-edna-mini\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `fastq.screen_taxonomy` / `kraken2` remains in governed FASTQ active scope because it is benchmark_ready"
        )
    }));
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "fastq.trim_reads\ttrimmomatic\tcorpus-01-mini\tcorpus_only\tfastq.adapter.trim_reads\tfastq.parser.trim_reads\tfastq_trim_reads_v2\tbenchmark_ready\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `fastq.trim_reads` / `trimmomatic` remains in governed FASTQ active scope because it is benchmark_ready"
        )
    }));
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "fastq.index_reference\tbowtie2_build\treference-index-assets\treference_fasta+reference_index_output\tfastq.adapter.index_reference\tfastq.parser.index_reference\tfastq_index_reference_v1\tbenchmark_ready\tobserver_specialized_benchmark\trunnable\tcomparable\tasset:reference-index-assets\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `fastq.index_reference` / `bowtie2_build` remains in governed FASTQ active scope because it is benchmark_ready"
        )
    }));
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "fastq.profile_overrepresented_sequences\tfastqc\tcorpus-01-mini\tcorpus_only\tfastq.adapter.profile_overrepresented_sequences\tfastq.parser.profile_overrepresented_sequences\tfastq_profile_overrepresented_sequences_v1\tbenchmark_ready\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `fastq.profile_overrepresented_sequences` / `fastqc` remains in governed FASTQ active scope because it is benchmark_ready"
        )
    }));
    assert!(rows.iter().all(|row| {
        !row.contains("\tplanned_contract\t")
            && !row.contains("\tdeclared_only\t")
            && !row.contains("\tplanner_only\t")
            && !row.contains("\tfastq.report_qc\t")
    }));
}
