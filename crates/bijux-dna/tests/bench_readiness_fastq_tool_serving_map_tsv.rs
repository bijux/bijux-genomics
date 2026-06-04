#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_tool_serving_map_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-tool-serving-map"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/fastq-tool-serving-map.tsv");
    assert!(tsv_path.is_file(), "FASTQ tool serving map TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read FASTQ tool serving map");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status")
    );
    let rows = lines.collect::<Vec<_>>();
    assert!(!rows.is_empty(), "TSV must contain FASTQ tool rows");
    assert!(
        rows.iter().any(|row| {
            row == &"fastqc\tfastq.validate_reads\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini"
        }),
        "TSV must retain the governed fastqc validation row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bijux_dna\tfastq.detect_duplicates_premerge\tgoverned_execution\trunnable\tparse_normalized\tfixture:corpus-01-mini"
        }),
        "TSV must retain the governed detect-duplicates-premerge row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bijux_dna\tfastq.estimate_library_complexity_prealign\tplanned_contract\tdeclared_only\tnot_normalized\tplanner_only"
        }),
        "TSV must retain the planned estimate-library-complexity-prealign row"
    );
    for tool_id in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.validate_reads\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed validation row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row == &"fastqc\tfastq.detect_adapters\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini"
        }),
        "TSV must retain the fixture-backed detect-adapters row for fastqc"
    );
    for tool_id in ["seqfu", "seqkit", "seqkit_stats"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.profile_reads\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed profile-reads row for {tool_id}"
        );
    }
    for tool_id in ["fastp", "prinseq", "seqfu", "seqkit_stats"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.profile_read_lengths\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed profile-read-lengths row for {tool_id}"
        );
    }
    for tool_id in [
        "adapterremoval",
        "alientrimmer",
        "atropos",
        "bbduk",
        "cutadapt",
        "fastp",
        "fastx_clipper",
        "leehom",
        "prinseq",
        "seqkit",
        "skewer",
        "trim_galore",
        "trimmomatic",
    ] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.trim_reads\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed trim-reads row for {tool_id}"
        );
    }
    for tool_id in ["bbduk", "fastp", "prinseq", "seqkit"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.filter_reads\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed filter-reads row for {tool_id}"
        );
    }
    for tool_id in ["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "vsearch"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.merge_pairs\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed merge-pairs row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row == &"umi_tools\tfastq.extract_umis\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tplanner_only"
        }),
        "TSV must retain the governed extract-umis row for umi_tools"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"cutadapt\tfastq.normalize_primers\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-03-amplicon-mini"
        }),
        "TSV must retain the governed normalize-primers row for cutadapt"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vsearch\tfastq.remove_chimeras\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-03-amplicon-mini"
        }),
        "TSV must retain the governed remove-chimeras row for vsearch"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vsearch\tfastq.cluster_otus\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-03-amplicon-mini"
        }),
        "TSV must retain the governed cluster-otus row for vsearch"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"seqkit\tfastq.normalize_abundance\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tplanner_only"
        }),
        "TSV must retain the governed normalize-abundance row for seqkit"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"seqfu\tfastq.normalize_abundance\tplanned_contract\tdeclared_only\tnot_normalized\tplanner_only"
        }),
        "TSV must retain the planned normalize-abundance row for seqfu"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"dada2\tfastq.infer_asvs\tgoverned_execution\trunnable\tparse_normalized\tfixture:corpus-03-amplicon-mini"
        }),
        "TSV must retain the governed infer-asvs row for dada2"
    );
    for tool_id in ["bayeshammer", "lighter", "musket", "rcorrector"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.correct_errors\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed correct-errors row for {tool_id}"
        );
    }
    for tool_id in ["bowtie2_build", "star"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.index_reference\tobserver_specialized_benchmark\trunnable\tcomparable\tplanner_only"
                )
            }),
            "TSV must retain the governed index-reference row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row == &"sortmerna\tfastq.deplete_rrna\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
        }),
        "TSV must retain the governed deplete-rrna row for sortmerna"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bowtie2\tfastq.deplete_host\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
        }),
        "TSV must retain the governed deplete-host row for bowtie2"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bowtie2\tfastq.deplete_reference_contaminants\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
        }),
        "TSV must retain the governed contaminant-depletion row for bowtie2"
    );
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.screen_taxonomy\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-02-edna-mini"
                )
            }),
            "TSV must retain the governed taxonomy-screen row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row == &"diamond\tfastq.screen_taxonomy\tplanned_contract\tdeclared_only\tnot_normalized\tfixture:corpus-02-edna-mini"
        }),
        "TSV must retain the planned taxonomy-screen row for diamond"
    );
    for tool_id in ["bbduk", "prinseq"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.filter_low_complexity\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tplanner_only"
                )
            }),
            "TSV must retain the governed filter-low-complexity row for {tool_id}"
        );
    }
    for tool_id in ["dustmasker", "fastp"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.filter_low_complexity\tplanned_contract\tdeclared_only\tnot_normalized\tplanner_only"
                )
            }),
            "TSV must retain the planned filter-low-complexity row for {tool_id}"
        );
    }
    for tool_id in ["bbduk", "fastp"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.trim_polyg_tails\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed trim-polyg row for {tool_id}"
        );
    }
    for tool_id in ["adapterremoval", "cutadapt", "seqkit"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.trim_terminal_damage\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed trim-terminal-damage row for {tool_id}"
        );
    }
    for tool_id in ["clumpify", "fastuniq"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.remove_duplicates\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tplanner_only"
                )
            }),
            "TSV must retain the governed remove-duplicates row for {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row == &"seqpurge\tfastq.trim_reads\tplanned_contract\tdeclared_only\tnot_normalized\tfixture:corpus-01-mini"
        }),
        "TSV must retain the planned seqpurge trim-reads row"
    );
}
