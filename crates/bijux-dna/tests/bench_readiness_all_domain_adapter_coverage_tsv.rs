#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_adapter_coverage_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-adapter-coverage"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domains/adapter-coverage.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain adapter coverage");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\treadiness_kind\tcommand_source\tcommand_step_count\tscript_command_count\tcommand_step_ids\tprimary_executables\tcoverage_status\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 124);
    assert!(rows.iter().any(|row| {
        row == &"fastq:corpus-01-mini:fastq.trim_reads:sample-set:trimmomatic\tfastq\tfastq.trim_reads\ttrimmomatic\tcorpus-01-mini\tcorpus_only\tfastq.adapter.trim_reads\tsmoke\tfastq_bam_command_adapter\t1\t1\tinvoke\tsh\tcovered\tactive row `fastq` / `fastq.trim_reads` / `trimmomatic` keeps executable command rendering through `fastq_bam_command_adapter` with 1 command step(s)"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:schmutzi\tbam\tbam.contamination\tschmutzi\tcorpus-01-adna-bam-mini\treference_fasta+reference_panel\tbam.adapter.contamination\tdry_or_smoke\tfastq_bam_command_adapter\t1\t1\tinvoke\t/bin/sh\tcovered\tactive row `bam` / `bam.contamination` / `schmutzi` keeps executable command rendering through `fastq_bam_command_adapter` with 1 command step(s)"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools\tvcf\tvcf.postprocess\tbcftools\tvcf_production_regression\tvcf_single_sample\tvcf.adapter.transform\tbenchmark_ready\tvcf_bcftools_adapter\t2\t2\tfill_tags,index_postprocess_vcf\tbcftools,bcftools\tcovered\tactive row `vcf` / `vcf.postprocess` / `bcftools` keeps executable command rendering through `vcf_bcftools_adapter` with 2 command step(s)"
    }));
    assert!(
        rows.iter().all(|row| row.contains("\tcovered\t")),
        "every active binding must keep rendered executable command coverage"
    );
}
