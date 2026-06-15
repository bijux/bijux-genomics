#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_adapter_output_contract_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-adapter-output-contract"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/fastq-adapter-output-contract.tsv");
    assert!(tsv_path.is_file(), "FASTQ adapter output contract TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read FASTQ adapter output contract");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 73, "TSV must retain the governed FASTQ 73-row slice");
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "seqkit_stats\tfastq.profile_reads\trunnable\tcomplete\tqc_json,qc_tsv,qc_plots_dir\tqc_json,qc_tsv,qc_plots_dir\t"
            )
        }),
        "TSV must retain the governed profile-reads contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "fastqc\tfastq.detect_adapters\trunnable\tcomplete\treport_json,adapter_report,adapter_evidence_dir\treport_json,adapter_report,adapter_evidence_dir\t"
            )
        }),
        "TSV must retain the governed detect-adapters contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "pear\tfastq.merge_pairs\trunnable\tcomplete\tmerged_reads,unmerged_reads_r1,unmerged_reads_r2,report_json,raw_backend_report_txt\tmerged_reads,unmerged_reads_r1,unmerged_reads_r2,report_json,raw_backend_report_txt\t"
            ) && row.contains("\treport_json\t")
        }),
        "TSV must retain the governed merge-pairs contract row for pear"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "umi_tools\tfastq.extract_umis\trunnable\tcomplete\tumi_reads_r1,umi_reads_r2,report_json\tumi_reads_r1,umi_reads_r2,report_json\t"
            ) && row.contains("\treport_json\t")
        }),
        "TSV must retain the governed extract-umis contract row for umi_tools"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "bijux_dna\tfastq.detect_duplicates_premerge\trunnable\tcomplete\tduplicate_signal_report\tduplicate_signal_report\tduplicate_signal_report,library_complexity_report\tduplicate_signal_report\tduplicate_signal_report\tduplicate_signal_report\t"
            )
        }),
        "TSV must retain the governed duplicate-signal output-contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "bijux_dna\tfastq.estimate_library_complexity_prealign\trunnable\tcomplete\tlibrary_complexity_report\tlibrary_complexity_report\tduplicate_signal_report,library_complexity_report\tlibrary_complexity_report\tlibrary_complexity_report\tlibrary_complexity_report\t"
            )
        }),
        "TSV must retain the governed estimate-library-complexity adapter contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "cutadapt\tfastq.trim_terminal_damage\trunnable\tcomplete\ttrimmed_reads_r1,trimmed_reads_r2,report_json,raw_backend_report_json\ttrimmed_reads_r1,trimmed_reads_r2,report_json,raw_backend_report_json\t"
            ) && row.contains("\treport_json\t")
        }),
        "TSV must retain the governed trim-terminal-damage contract row for cutadapt"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with("bowtie2\tfastq.deplete_host\trunnable\tcomplete\t")
                && row.contains("\thost_depletion_report_json\t")
        }),
        "TSV must retain the governed bowtie2 host-depletion contract row"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("diamond\tfastq.screen_taxonomy\t")),
        "TSV must not retain removed diamond taxonomy output-contract rows"
    );
}
