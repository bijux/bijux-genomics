#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_output_declarations_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-output-declarations"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/output-declarations-all-domains.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain output declarations");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\traw_outputs\tnormalized_metrics\tlogs\tmanifest\tindex_outputs\tstatus"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert!(rows.len() >= 128);

    let has_row = |result_id: &str,
                   normalized_metric: &str,
                   raw_output: &str,
                   manifest_suffix: &str,
                   index_output: &str| {
        rows.iter().any(|row| {
            let columns = row.split('\t').collect::<Vec<_>>();
            columns.len() == 12
                && columns[0] == result_id
                && columns[6].split(',').any(|value| value == raw_output)
                && columns[7].split(',').any(|value| value == normalized_metric)
                && columns[8].contains("stdout=")
                && columns[8].contains("stderr=")
                && columns[9].ends_with(manifest_suffix)
                && (index_output.is_empty()
                    || columns[10].split(',').any(|value| value == index_output))
                && columns[11] == "complete"
        })
    };

    assert!(
        has_row(
            "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
            "classification_report_json",
            "screen_report_tsv",
            "/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json",
            "",
        ),
        "taxonomy row must keep raw outputs, normalized metrics, logs, and manifest explicit"
    );
    assert!(
        has_row(
            "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
            "kinship_report",
            "summary",
            "/bam.kinship/sample-set/king/stage-result.json",
            "",
        ),
        "kinship row must keep raw outputs, normalized metrics, logs, and manifest explicit"
    );
    assert!(
        has_row(
            "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
            "called_vcf",
            "called_vcf",
            "/vcf.call/bam_bundle/bcftools/stage-result.json",
            "called_vcf_tbi",
        ),
        "VCF call row must keep raw outputs, normalized metrics, logs, manifest, and index outputs explicit"
    );
    assert!(
        has_row(
            "vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle",
            "imputation_metrics_json",
            "imputation_metrics_json",
            "/vcf.imputation_metrics/vcf_cohort_with_panel/beagle/stage-result.json",
            "",
        ),
        "VCF imputation metrics row must keep raw outputs, normalized metrics, logs, and manifest explicit"
    );
}
