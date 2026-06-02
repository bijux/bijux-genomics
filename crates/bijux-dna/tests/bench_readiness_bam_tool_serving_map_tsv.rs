#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_tool_serving_map_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-tool-serving-map"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/bam-tool-serving-map.tsv");
    assert!(tsv_path.is_file(), "BAM tool serving map TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read BAM tool serving map");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 45, "TSV must retain the governed BAM row count");
    assert!(
        rows.iter().any(|row| {
            row == &"bedtools\tbam.coverage\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed bedtools coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bwa\tbam.align\tsupported\trunnable\tartifact_contract_only\tfixture:corpus-01-mini"
        }),
        "TSV must retain the governed bwa alignment row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bamtools\tbam.filter\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed bamtools filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bamtools\tbam.mapq_filter\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed bamtools MAPQ-filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bedtools\tbam.filter\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed bedtools filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.endogenous_content\tsupported\tplannable\tscientific_report_contract\tplanner_only"
        }),
        "TSV must retain the governed samtools endogenous-content row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.filter\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed samtools filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.mapq_filter\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed samtools MAPQ-filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"picard\tbam.insert_size\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed picard insert-size row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"picard\tbam.gc_bias\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed picard GC-bias row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"picard\tbam.duplication_metrics\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed picard duplication-metrics row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.duplication_metrics\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed samtools duplication-metrics row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"preseq\tbam.complexity\tplanned\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the planned preseq complexity row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"picard\tbam.markdup\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed picard markdup row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.markdup\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed samtools markdup row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.length_filter\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed samtools length-filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"picard\tbam.length_filter\tsupported\tplannable\tartifact_contract_only\tplanner_only"
        }),
        "TSV must retain the governed picard length-filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bamtools\tbam.validate\tsupported\tplannable\tparser_fixture_validated\tfixture:corpus-01-bam-mini"
        }),
        "TSV must retain the governed bamtools validation row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bedtools\tbam.validate\tsupported\tplannable\tparser_fixture_validated\tfixture:corpus-01-bam-mini"
        }),
        "TSV must retain the governed bedtools validation row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.validate\tsupported\tplannable\tparser_fixture_validated\tfixture:corpus-01-bam-mini"
        }),
        "TSV must retain the governed samtools validation row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"multiqc\tbam.qc_pre\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed multiqc qc_pre reporting row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.mapping_summary\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed samtools mapping-summary row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"picard\tbam.mapping_summary\tsupported\tplannable\tparser_fixture_validated\tplanner_only"
        }),
        "TSV must retain the governed picard mapping-summary row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bcftools\tbam.genotyping\tmissing_contract\tdeclared_only\tartifact_contract_only\tplanner_only"
        }),
        "TSV must surface missing BAM tool contracts explicitly"
    );
}
