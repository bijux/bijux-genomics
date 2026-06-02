#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_unregistered_benchmark_pairs_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-unregistered-benchmark-pairs"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/unregistered-benchmark-pairs.tsv");
    assert!(tsv_path.is_file(), "unregistered benchmark pairs TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read unregistered benchmark pairs");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\ttool_id\tsupport_status\tregistry_status\tregistered_stage_ids\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 15, "TSV must retain the governed unregistered-pair row count");
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.genotyping\tangsd\tplanned\ttool_registered_pair_missing\tbam.kinship,bam.sex\tbenchmark matrix references `bam.genotyping` / `angsd` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_registered_pair_missing; registered stages for `angsd`: bam.kinship, bam.sex"
        }),
        "TSV must retain the governed bam.genotyping / angsd registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.detect_duplicates_premerge\tbijux_dna\tplanned_contract\ttool_missing\t\tbenchmark matrix references `fastq.detect_duplicates_premerge` / `bijux_dna` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_missing; registered stages for `bijux_dna`: <none>"
        }),
        "TSV must retain the governed fastq.detect_duplicates_premerge / bijux_dna registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.estimate_library_complexity_prealign\tbijux_dna\tplanned_contract\ttool_missing\t\tbenchmark matrix references `fastq.estimate_library_complexity_prealign` / `bijux_dna` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_missing; registered stages for `bijux_dna`: <none>"
        }),
        "TSV must retain the governed fastq.estimate_library_complexity_prealign / bijux_dna registry drift row"
    );
    for tool_id in ["fastp", "prinseq", "seqfu"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.profile_read_lengths\t{tool_id}\t"))
            }),
            "TSV must no longer retain a registry-drift row for fastq.profile_read_lengths / {tool_id}"
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
            !rows
                .iter()
                .any(|row| { row.starts_with(&format!("fastq\tfastq.trim_reads\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for fastq.trim_reads / {tool_id}"
        );
    }
    for tool_id in ["bbduk", "fastp", "prinseq", "seqkit"] {
        assert!(
            !rows
                .iter()
                .any(|row| { row.starts_with(&format!("fastq\tfastq.filter_reads\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for fastq.filter_reads / {tool_id}"
        );
    }
    for tool_id in ["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "vsearch"] {
        assert!(
            !rows
                .iter()
                .any(|row| { row.starts_with(&format!("fastq\tfastq.merge_pairs\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for fastq.merge_pairs / {tool_id}"
        );
    }
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.extract_umis\tumi_tools\t") }),
        "TSV must not retain a registry-drift row for fastq.extract_umis / umi_tools"
    );
    for tool_id in ["bamtools", "bedtools", "samtools"] {
        assert!(
            !rows.iter().any(|row| { row.starts_with(&format!("bam\tbam.validate\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for bam.validate / {tool_id}"
        );
    }
    assert!(
        !rows.iter().any(|row| { row.starts_with("bam\tbam.qc_pre\tmultiqc\t") }),
        "TSV must not retain a registry-drift row for bam.qc_pre / multiqc"
    );
    for tool_id in ["bwa", "bowtie2"] {
        assert!(
            !rows.iter().any(|row| { row.starts_with(&format!("bam\tbam.align\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for bam.align / {tool_id}"
        );
    }
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.normalize_primers\tcutadapt\t") }),
        "TSV must not retain a registry-drift row for fastq.normalize_primers / cutadapt"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.remove_chimeras\tvsearch\t") }),
        "TSV must not retain a registry-drift row for fastq.remove_chimeras / vsearch"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.cluster_otus\tvsearch\t") }),
        "TSV must not retain a registry-drift row for fastq.cluster_otus / vsearch"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.normalize_abundance\tseqkit\t") }),
        "TSV must not retain a registry-drift row for fastq.normalize_abundance / seqkit"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "fastq\tfastq.normalize_abundance\tseqfu\tplanned_contract\ttool_registered_pair_missing\t",
            )
        }),
        "TSV must retain the planned fastq.normalize_abundance / seqfu registry-drift row"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.infer_asvs\tdada2\t") }),
        "TSV must not retain a registry-drift row for fastq.infer_asvs / dada2"
    );
    for tool_id in ["bayeshammer", "lighter", "musket", "rcorrector"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.correct_errors\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for fastq.correct_errors / {tool_id}"
        );
    }
    for tool_id in ["bowtie2_build", "star"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.index_reference\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for fastq.index_reference / {tool_id}"
        );
    }
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.deplete_rrna\tsortmerna\t") }),
        "TSV must not retain a registry-drift row for fastq.deplete_rrna / sortmerna"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("fastq\tfastq.deplete_host\tbowtie2\t") }),
        "TSV must not retain a registry-drift row for fastq.deplete_host / bowtie2"
    );
    assert!(
        !rows.iter().any(|row| {
            row.starts_with("fastq\tfastq.deplete_reference_contaminants\tbowtie2\t")
        }),
        "TSV must not retain a registry-drift row for fastq.deplete_reference_contaminants / bowtie2"
    );
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.screen_taxonomy\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for fastq.screen_taxonomy / {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "fastq\tfastq.screen_taxonomy\tdiamond\tplanned_contract\ttool_missing\t",
            )
        }),
        "TSV must retain the planned fastq.screen_taxonomy / diamond registry-drift row"
    );
    for tool_id in ["bbduk", "prinseq"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.filter_low_complexity\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for fastq.filter_low_complexity / {tool_id}"
        );
    }
    for tool_id in ["dustmasker", "fastp"] {
        assert!(
            rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.filter_low_complexity\t{tool_id}\t"))
            }),
            "TSV must retain the planned fastq.filter_low_complexity / {tool_id} registry-drift row"
        );
    }
    for tool_id in ["bbduk", "fastp"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.trim_polyg_tails\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for fastq.trim_polyg_tails / {tool_id}"
        );
    }
    for tool_id in ["adapterremoval", "cutadapt", "seqkit"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.trim_terminal_damage\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for fastq.trim_terminal_damage / {tool_id}"
        );
    }
    for tool_id in ["clumpify", "fastuniq"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("fastq\tfastq.remove_duplicates\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for fastq.remove_duplicates / {tool_id}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row.starts_with("fastq\tfastq.trim_reads\tseqpurge\tplanned_contract\ttool_missing\t")
        }),
        "TSV must retain the planned fastq.trim_reads / seqpurge registry-drift row"
    );
}
