#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_local_job_coverage_writes_governed_tsv_file() {
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domains/local-job-coverage.tsv");

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
    assert_eq!(rows.len(), 125);
    assert!(rows.iter().any(|row| {
        row == &"fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2\tfastq\tfastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tfastq.adapter.screen_taxonomy\tfastq.parser.screen_taxonomy\tfastq_screen_taxonomy_v1\tdry_or_smoke\tbenchmark_ready\tfastq_bam_command_adapter\t1\t1\tinvoke\tsh\tbenchmarks/readiness/rendered-commands-all-domains.sh\tbenchmarks/readiness/rendered-commands-all-domains.argv.jsonl\tcovered\tactive row `fastq` / `fastq.screen_taxonomy` / `kraken2` keeps one local benchmark job row in `benchmarks/readiness/rendered-commands-all-domains.sh` and `benchmarks/readiness/rendered-commands-all-domains.argv.jsonl` through `fastq_bam_command_adapter`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam:corpus-01-kinship-mini:bam.kinship:sample-set:king\tbam\tbam.kinship\tking\tcorpus-01-kinship-mini\treference_fasta+reference_panel\tbam.adapter.kinship\tbam.parser.kinship\tbam_kinship_normalized_v1\tsmoke\tbenchmark_ready\tfastq_bam_command_adapter\t1\t1\tinvoke\tcargo\tbenchmarks/readiness/rendered-commands-all-domains.sh\tbenchmarks/readiness/rendered-commands-all-domains.argv.jsonl\tcovered\tactive row `bam` / `bam.kinship` / `king` keeps one local benchmark job row in `benchmarks/readiness/rendered-commands-all-domains.sh` and `benchmarks/readiness/rendered-commands-all-domains.argv.jsonl` through `fastq_bam_command_adapter`"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools\tvcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tvcf.adapter.transform\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.postprocess.v1\tbenchmark_ready\tbenchmark_ready\tvcf_bcftools_adapter\t2\t2\tfill_tags,index_postprocess_vcf\tbcftools,bcftools\tbenchmarks/readiness/rendered-commands-all-domains.sh\tbenchmarks/readiness/rendered-commands-all-domains.argv.jsonl\tcovered\tactive row `vcf` / `vcf.postprocess` / `bcftools` keeps one local benchmark job row in `benchmarks/readiness/rendered-commands-all-domains.sh` and `benchmarks/readiness/rendered-commands-all-domains.argv.jsonl` through `vcf_bcftools_adapter`"
    }));
    assert!(
        rows.iter().all(|row| row.contains("\tcovered\t")),
        "every active binding must retain complete governed local benchmark job coverage"
    );
}
