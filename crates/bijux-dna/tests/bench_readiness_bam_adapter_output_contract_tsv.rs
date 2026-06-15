#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_adapter_output_contract_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-adapter-output-contract"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/bam-adapter-output-contract.tsv");
    assert!(tsv_path.is_file(), "BAM adapter output contract TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read BAM adapter output contract");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 49, "TSV must retain the governed BAM 49-row slice");
    let has_row = |tool_id: &str,
                   stage_id: &str,
                   adapter_status: &str,
                   output_contract_status: &str,
                   normalized_metrics_output_id: &str,
                   missing_declarations: &str| {
        rows.iter().any(|row| {
            let columns = row.split('\t').collect::<Vec<_>>();
            columns.len() == 15
                && columns[0] == tool_id
                && columns[1] == stage_id
                && columns[2] == adapter_status
                && columns[3] == output_contract_status
                && columns[9] == normalized_metrics_output_id
                && columns[13] == missing_declarations
        })
    };
    assert!(
        has_row("samtools", "bam.validate", "runnable", "complete", "validation_report", "",),
        "TSV must retain the governed samtools validate contract row"
    );
    assert!(
        has_row(
            "authenticct",
            "bam.authenticity",
            "runnable",
            "complete",
            "authenticity_report",
            "",
        ),
        "TSV must retain the governed authenticct authenticity contract row"
    );
    assert!(
        has_row("bowtie2", "bam.align", "runnable", "complete", "align_metrics", ""),
        "TSV must retain the governed bowtie2 alignment contract row"
    );
    assert!(
        has_row("rxy", "bam.sex", "runnable", "complete", "sex_report", ""),
        "TSV must retain the governed rxy bam.sex contract row"
    );
    assert!(
        has_row("yleaf", "bam.sex", "runnable", "complete", "sex_report", ""),
        "TSV must retain the governed yleaf bam.sex contract row"
    );
    assert!(
        has_row("yleaf", "bam.haplogroups", "runnable", "complete", "haplogroups", ""),
        "TSV must retain the governed yleaf bam.haplogroups contract row"
    );
    assert!(
        has_row("bamutil", "bam.overlap_correction", "runnable", "complete", "summary", ""),
        "TSV must retain the governed bamutil overlap-correction contract row"
    );
    assert!(
        has_row("angsd", "bam.genotyping", "runnable", "complete", "genotyping_report", "",),
        "TSV must retain the governed angsd bam.genotyping contract row"
    );
}
