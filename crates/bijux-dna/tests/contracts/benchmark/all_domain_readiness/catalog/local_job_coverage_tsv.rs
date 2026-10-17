#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

#[test]
fn bench_readiness_all_domain_local_job_coverage_writes_governed_tsv_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-all-domain-local-job-coverage"])
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
        "artifacts/benchmark-smoke/readiness/all-domains/local-job-coverage.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read local job coverage TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\treadiness_kind\tbenchmark_status\tcommand_source\tcommand_step_count\tscript_command_count\tcommand_step_ids\tprimary_executables\tscript_output_path\targv_output_path\tcoverage_status\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert!(rows.len() >= 128);
    let script_path =
        "artifacts/benchmark-smoke/readiness/all-domains/rendered-commands-all-domains.sh";
    let argv_path =
        "artifacts/benchmark-smoke/readiness/all-domains/rendered-commands-all-domains.argv.jsonl";
    for (result_id, adapter_id, parser_id, command_source, command_steps, executables) in [
        (
            "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
            "fastq.adapter.screen_taxonomy",
            "fastq.parser.screen_taxonomy",
            "fastq_bam_command_adapter",
            "invoke",
            "sh",
        ),
        (
            "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
            "bam.adapter.kinship",
            "bam.parser.kinship",
            "fastq_bam_command_adapter",
            "invoke",
            "cargo",
        ),
        (
            "vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools",
            "vcf.adapter.transform",
            "vcf.parser.vcf_output",
            "vcf_bcftools_adapter",
            "fill_tags,index_postprocess_vcf",
            "bcftools,bcftools",
        ),
        (
            "vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle",
            "vcf.adapter.panel_workflow",
            "vcf.parser.report_json",
            "vcf_imputation_family_adapter",
            "impute,index_imputed_vcf,derive_imputation_metrics",
            "sh,bcftools,sh",
        ),
    ] {
        let columns = rows
            .iter()
            .find(|row| row.starts_with(&format!("{result_id}\t")))
            .map(|row| row.split('\t').collect::<Vec<_>>())
            .expect("governed local-job coverage row");
        assert_eq!(columns[6], adapter_id);
        assert_eq!(columns[7], parser_id);
        assert_eq!(columns[11], command_source);
        assert_eq!(columns[14], command_steps);
        assert_eq!(columns[15], executables);
        assert_eq!(columns[16], script_path);
        assert_eq!(columns[17], argv_path);
        assert_eq!(columns[18], "covered");
        assert!(columns[19].contains(script_path));
        assert!(columns[19].contains(argv_path));
    }
    assert!(
        rows.iter().all(|row| row.contains("\tcovered\t")),
        "every active binding must retain complete governed local benchmark job coverage"
    );
}
