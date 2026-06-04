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
    assert_eq!(rows.len(), 51, "TSV must retain the governed BAM row count");
    for row in [
        "addeam\tbam.damage\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-adna-damage-mini",
        "damageprofiler\tbam.damage\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-adna-damage-mini",
        "bwa\tbam.align\tsupported\trunnable\tartifact_contract_only\tfixture:corpus-01-mini",
        "bowtie2\tbam.align\tsupported\trunnable\tartifact_contract_only\tfixture:corpus-01-mini",
        "bamtools\tbam.validate\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "multiqc\tbam.qc_pre\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "samtools\tbam.qc_pre\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "samtools\tbam.mapping_summary\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "picard\tbam.mapping_summary\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "samtools\tbam.filter\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "bamtools\tbam.filter\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "bedtools\tbam.filter\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini",
        "samtools\tbam.bias_mitigation\tmismatched_contract\tdeclared_only\tartifact_contract_only\tplanner_only",
        "samtools\tbam.haplogroups\tmismatched_contract\tdeclared_only\tscientific_report_contract\tplanner_only",
        "yleaf\tbam.haplogroups\tsupported\trunnable\tscientific_report_contract\tplanner_only",
    ] {
        assert!(
            rows.iter().any(|candidate| candidate == &row),
            "TSV must retain governed BAM readiness row: {row}"
        );
    }
    assert!(
        !rows.iter().any(|row| row == &"samtools\tbam.align\tsupported\trunnable\tartifact_contract_only\tfixture:corpus-01-mini"),
        "TSV must not retain a samtools alignment row once bam.align is limited to the admitted aligners"
    );
}
