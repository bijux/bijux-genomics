#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_output_contract_coverage_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-output-contract-coverage"])
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
        "benchmarks/readiness/all-domains/output-contract-coverage.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain output-contract coverage");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tproof_source\tsource_contract_status\toutput_declaration_status\traw_output_ids\tnormalized_metric_ids\tlog_declarations\tmanifest\tindex_output_ids\traw_outputs_declared\tnormalized_metrics_declared\tlogs_declared\tmanifest_declared\tindex_coverage_status\tcoverage_status\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 126);
    assert!(rows.iter().any(|row| {
        row == &"fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2\tfastq\tfastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tfastq.adapter.screen_taxonomy\tfastq_output_contract\tcomplete\tcomplete\tscreen_report_tsv,unclassified_reads_r1,unclassified_reads_r2\tclassification_report_json\tstdout=runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stdout.log,stderr=runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stderr.log\truns/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json\t\ttrue\ttrue\ttrue\ttrue\tnot_applicable\tcovered\tactive row `fastq` / `fastq.screen_taxonomy` / `kraken2` keeps governed raw outputs, normalized metrics, logs, and manifest declarations through `fastq_output_contract` with no index requirement"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam:corpus-01-kinship-mini:bam.kinship:sample-set:king\tbam\tbam.kinship\tking\tcorpus-01-kinship-mini\treference_fasta+reference_panel\tbam.adapter.kinship\tbam_output_contract\tcomplete\tcomplete\tsummary,stage_metrics\tkinship_report\tstdout=runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-kinship-mini/bam.kinship/sample-set/king/stdout.log,stderr=runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-kinship-mini/bam.kinship/sample-set/king/stderr.log\truns/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-kinship-mini/bam.kinship/sample-set/king/stage-result.json\t\ttrue\ttrue\ttrue\ttrue\tnot_applicable\tcovered\tactive row `bam` / `bam.kinship` / `king` keeps governed raw outputs, normalized metrics, logs, and manifest declarations through `bam_output_contract` with no index requirement"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools\tvcf\tvcf.call\tbcftools\tvcf_production_regression\tbam_bundle\tvcf.adapter.calling\tvcf_output_contract\tcomplete\tcomplete\tcalled_vcf\tcalled_vcf\tstdout=runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/vcf_production_regression/vcf.call/bam_bundle/bcftools/stdout.log,stderr=runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/vcf_production_regression/vcf.call/bam_bundle/bcftools/stderr.log\truns/bench/slurm-dry-run/runs/local-benchmark-dry-run/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json\tcalled_vcf_tbi\ttrue\ttrue\ttrue\ttrue\tcovered\tcovered\tactive row `vcf` / `vcf.call` / `bcftools` keeps governed raw outputs, normalized metrics, logs, manifest, and required index declarations through `vcf_output_contract`"
    }));
    assert!(
        rows.iter().all(|row| row.contains("\tcovered\t")),
        "every active binding must retain complete governed output-contract coverage"
    );
}
