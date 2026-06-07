#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_active_stage_catalog_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-active-stage-catalog"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domains/active-stage-catalog.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain active stage catalog");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\treadiness_kind\tactive_tool_count\tbenchmark_ready_tool_count\tparser_row_count\tparser_covered_row_count\tschema_present\treport_row_count\tbenchmark_statuses\tactive_tool_ids\tbenchmark_ready_tool_ids\treport_section_ids"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 56);
    assert!(rows.iter().any(|row| {
        row == &"bam\tbam.damage\tsmoke\t6\t6\t6\t6\ttrue\t1\tbenchmark_ready\taddeam,damageprofiler,mapdamage2,ngsbriggs,pmdtools,pydamage\taddeam,damageprofiler,mapdamage2,ngsbriggs,pmdtools,pydamage\tancient_signal"
    }));
    assert!(rows.iter().any(|row| {
        row == &"fastq\tfastq.trim_reads\tsmoke\t13\t13\t13\t13\ttrue\t1\tbenchmark_ready\tadapterremoval,alientrimmer,atropos,bbduk,cutadapt,fastp,fastx_clipper,leehom,prinseq,seqkit,skewer,trim_galore,trimmomatic\tadapterremoval,alientrimmer,atropos,bbduk,cutadapt,fastp,fastx_clipper,leehom,prinseq,seqkit,skewer,trim_galore,trimmomatic\tread_cleanup"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf\tvcf.stats\tsmoke\t1\t1\t1\t1\ttrue\t1\tbenchmark_ready\tbcftools\tbcftools\tquality_control"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf\tvcf.postprocess\tsmoke\t1\t1\t1\t1\ttrue\t1\tbenchmark_ready\tbcftools\tbcftools\tnormalization"
    }));
    assert!(
        rows.iter().all(|row| {
            !row.contains("\tfastq.index_reference\t")
                && !row.contains("\tfastq.profile_overrepresented_sequences\t")
                && !row.contains("\tfastq.report_qc\t")
        }),
        "active stage catalog TSV must exclude not-benchmark-ready-only stages"
    );
}
