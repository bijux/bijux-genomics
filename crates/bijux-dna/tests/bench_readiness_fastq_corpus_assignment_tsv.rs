#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_corpus_assignment_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-corpus-assignment"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/fastq-corpus-assignment.tsv");
    assert!(tsv_path.is_file(), "FASTQ corpus assignment TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read FASTQ corpus assignment");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tassignment_status\tcorpus_family_id\tfixture_id\tbenchmark_scope_id\texcluded_reason\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert!(!rows.is_empty(), "TSV must contain FASTQ corpus assignment rows");
    assert!(
        rows.iter().any(|row| {
            row == &"fastqc\tfastq.validate_reads\tbenchmark_ready\tobserver_specialized_benchmark\trunnable\tcomparable\tassigned\tcorpus-01\tcorpus-01-mini\t\t\trow `fastq.validate_reads` / `fastqc` is benchmark_ready and maps to `corpus-01` via fixture `corpus-01-mini`: General FASTQ preprocessing and screening stages stay on the governed corpus-01 slice for local benchmark comparability."
        }),
        "TSV must retain the governed corpus-01 validation row"
    );
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.screen_taxonomy\tbenchmark_ready\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tassigned\tcorpus-02\tcorpus-02-edna-mini\t\t\trow `fastq.screen_taxonomy` / `{tool_id}` is benchmark_ready and maps to `corpus-02` via fixture `corpus-02-edna-mini`: Taxonomy screening requires the governed eDNA mock-community corpus rather than the general shotgun slice."
                )
            }),
            "TSV must retain the governed corpus-02 taxonomy row for `{tool_id}`"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row == &"cutadapt\tfastq.normalize_primers\tbenchmark_ready\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tassigned\tcorpus-03\tcorpus-03-amplicon-mini\t\t\trow `fastq.normalize_primers` / `cutadapt` is benchmark_ready and maps to `corpus-03` via fixture `corpus-03-amplicon-mini`: Amplicon-oriented stages must run on the governed amplicon corpus to preserve primer and ASV or OTU semantics."
        }),
        "TSV must retain the governed corpus-03 amplicon row"
    );
    for (tool_id, stage_id, benchmark_status, support_status, adapter_status, parser_status) in [
        (
            "vsearch",
            "fastq.remove_chimeras",
            "benchmark_ready",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "dada2",
            "fastq.infer_asvs",
            "benchmark_ready",
            "governed_execution",
            "runnable",
            "parse_normalized",
        ),
        (
            "vsearch",
            "fastq.cluster_otus",
            "benchmark_ready",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "seqkit",
            "fastq.normalize_abundance",
            "benchmark_ready",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
    ] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\t{stage_id}\t{benchmark_status}\t{support_status}\t{adapter_status}\t{parser_status}\tassigned\tcorpus-03\tcorpus-03-amplicon-mini\t\t\trow `{stage_id}` / `{tool_id}` is {benchmark_status} and maps to `corpus-03` via fixture `corpus-03-amplicon-mini`: Amplicon-oriented stages must run on the governed amplicon corpus to preserve primer and ASV or OTU semantics."
                )
            }),
            "TSV must retain the governed corpus-03 amplicon row for `{stage_id}` / `{tool_id}`"
        );
    }
    assert!(
        !rows.iter().any(|row| row.starts_with("seqfu\tfastq.normalize_abundance\t")),
        "TSV must not retain a corpus assignment row for seqfu normalize-abundance"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bowtie2_build\tfastq.index_reference\tbenchmark_ready\tobserver_specialized_benchmark\trunnable\tcomparable\tasset_backed\t\t\treference-index-assets\t\trow `fastq.index_reference` / `bowtie2_build` is benchmark_ready and maps to asset-backed benchmark scope `reference-index-assets`: Reference indexing is benchmarked against a governed reference-asset scope rather than a read corpus because it prepares reusable index artifacts."
        }),
        "TSV must retain the explicit index-reference asset-backed scope"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastqc\tfastq.profile_overrepresented_sequences\tbenchmark_ready\tobserver_specialized_benchmark\trunnable\tcomparable\tassigned\tcorpus-01\tcorpus-01-mini\t\t\trow `fastq.profile_overrepresented_sequences` / `fastqc` is benchmark_ready and maps to `corpus-01` via fixture `corpus-01-mini`: General FASTQ preprocessing and screening stages stay on the governed corpus-01 slice for local benchmark comparability."
        }),
        "TSV must retain the governed overrepresented-sequence corpus assignment"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"multiqc\tfastq.report_qc\tnot_benchmark_ready\tobserver_specialized_benchmark\trunnable\tcomparable\texcluded\t\t\t\tgoverned_multiqc_bundle_fixture_missing\trow `fastq.report_qc` / `multiqc` is not_benchmark_ready and remains excluded from governed corpus assignment: The governed QC-report bundle is not yet owned by a corpus fixture with reviewer-stable benchmark expectations."
        }),
        "TSV must retain the explicit report-qc exclusion"
    );
}
