#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_expected_benchmark_results_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-expected-benchmark-results"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/expected-benchmark-results.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read expected benchmark results TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_row_id\tdomain\tstage_id\ttool_id\treadiness_kind\tcorpus_family_id\tfixture_id\tsample_scope\texpected_output_artifact_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tresult_root\tstage_result_manifest_path\tstdout_path\tstderr_path\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 112, "TSV must retain every benchmark-ready expected result row");
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2\tfastq\tfastq.screen_taxonomy\tkraken2\t"
            ) && row.contains(
                "\tdry_or_smoke\tcorpus-02\tcorpus-02-edna-mini\tsample-set\tclassification_report_json,screen_report_tsv,unclassified_reads_r1,unclassified_reads_r2\t"
            ) && row.contains(
                "\tscreen_report_tsv,unclassified_reads_r1,unclassified_reads_r2\tclassification_report_json\ttarget/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2\t"
            )
        }),
        "TSV must retain the governed taxonomy expected-result row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:ngsbriggs\tbam\tbam.damage\tngsbriggs\t"
            ) && row.contains(
                "\tsmoke\tcorpus-01-adna-bam\tcorpus-01-adna-damage-mini\tadna_damage_non_udg\t"
            ) && row.contains(
                "\tdamage_report,terminal_position_metrics,parser_output,stage_metrics,damage_profile,damage_parameters\tterminal_position_metrics,parser_output,stage_metrics,damage_profile,damage_parameters\tdamage_report\ttarget/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-adna-damage-mini/bam.damage/adna_damage_non_udg/ngsbriggs\t"
            )
        }),
        "TSV must retain the governed ancient-DNA damage expected-result row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king\tbam\tbam.kinship\tking\t"
            ) && row.contains(
                "\tsmoke\tcorpus-01-kinship\tcorpus-01-kinship-mini\tsample-set\tkinship_report,summary,stage_metrics\t"
            ) && row.contains(
                "\tsummary,stage_metrics\tkinship_report\ttarget/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-kinship-mini/bam.kinship/sample-set/king\t"
            )
        }),
        "TSV must retain the governed kinship expected-result row"
    );
}
