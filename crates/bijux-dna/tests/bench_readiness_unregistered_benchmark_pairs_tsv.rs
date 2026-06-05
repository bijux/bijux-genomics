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
    assert_eq!(rows.len(), 7, "TSV must retain the governed unregistered-pair row count");
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.genotyping\tbcftools\tmissing_contract\ttool_missing\t\tbenchmark matrix references `bam.genotyping` / `bcftools` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_missing; registered stages for `bcftools`: <none>"
        }),
        "TSV must retain the governed bam.genotyping / bcftools registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.genotyping\tangsd\tplanned\ttool_registered_pair_missing\tbam.kinship,bam.sex\tbenchmark matrix references `bam.genotyping` / `angsd` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_registered_pair_missing; registered stages for `angsd`: bam.kinship, bam.sex"
        }),
        "TSV must retain the governed bam.genotyping / angsd registry drift row"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.damage\tngsbriggs\t")),
        "TSV must not retain a registry-drift row for bam.damage / ngsbriggs once it is registered"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.recalibration\tgatk\t")),
        "TSV must not retain a registry-drift row for bam.recalibration / gatk once it is registered in production"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.estimate_library_complexity_prealign\tbijux_dna\tplanned_contract\ttool_registered_pair_missing\tfastq.detect_duplicates_premerge\tbenchmark matrix references `fastq.estimate_library_complexity_prealign` / `bijux_dna` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_registered_pair_missing; registered stages for `bijux_dna`: fastq.detect_duplicates_premerge"
        }),
        "TSV must retain the governed fastq.estimate_library_complexity_prealign / bijux_dna registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.filter_low_complexity\tdustmasker\tplanned_contract\ttool_missing\t\tbenchmark matrix references `fastq.filter_low_complexity` / `dustmasker` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_missing; registered stages for `dustmasker`: <none>"
        }),
        "TSV must retain the governed fastq.filter_low_complexity / dustmasker registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.filter_low_complexity\tfastp\tplanned_contract\ttool_registered_pair_missing\tfastq.filter_reads,fastq.profile_read_lengths,fastq.trim_polyg_tails,fastq.trim_reads\tbenchmark matrix references `fastq.filter_low_complexity` / `fastp` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_registered_pair_missing; registered stages for `fastp`: fastq.filter_reads, fastq.profile_read_lengths, fastq.trim_polyg_tails, fastq.trim_reads"
        }),
        "TSV must retain the governed fastq.filter_low_complexity / fastp registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.trim_reads\tseqpurge\tplanned_contract\ttool_missing\t\tbenchmark matrix references `fastq.trim_reads` / `seqpurge` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_missing; registered stages for `seqpurge`: <none>"
        }),
        "TSV must retain the governed fastq.trim_reads / seqpurge registry drift row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq\tfastq.normalize_abundance\tseqfu\tplanned_contract\ttool_registered_pair_missing\tfastq.profile_read_lengths,fastq.profile_reads\tbenchmark matrix references `fastq.normalize_abundance` / `seqfu` but configs/ci/registry/tool_registry.toml does not register that pair; registry status: tool_registered_pair_missing; registered stages for `seqfu`: fastq.profile_read_lengths, fastq.profile_reads"
        }),
        "TSV must retain the governed fastq.normalize_abundance / seqfu registry drift row"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.complexity\t")),
        "TSV must not retain a registry-drift row for bam.complexity once preseq is registered in production"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.align\t")),
        "TSV must not retain a registry-drift row for bam.align once both admitted aligners are registered"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.overlap_correction\t")),
        "TSV must not retain a registry-drift row for bam.overlap_correction once bamutil is registered in production"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.haplogroups\t")),
        "TSV must not retain a registry-drift row for bam.haplogroups once yleaf is registered in production"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.damage\taddeam\t")),
        "TSV must not retain a registry-drift row for bam.damage / addeam once it is registered"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.damage\tdamageprofiler\t")),
        "TSV must not retain a registry-drift row for bam.damage / damageprofiler once it is registered"
    );
    for tool_id in ["prinseq", "seqfu"] {
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
        !rows.iter().any(|row| { row.starts_with("bam\tbam.endogenous_content\tsamtools\t") }),
        "TSV must not retain a registry-drift row for bam.endogenous_content / samtools"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("bam\tbam.qc_pre\tmultiqc\t") }),
        "TSV must not retain a registry-drift row for bam.qc_pre / multiqc"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("bam\tbam.mapping_summary\tpicard\t") }),
        "TSV must not retain a registry-drift row for bam.mapping_summary / picard"
    );
    for tool_id in ["bamtools", "bedtools", "samtools"] {
        assert!(
            !rows.iter().any(|row| { row.starts_with(&format!("bam\tbam.filter\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for bam.filter / {tool_id}"
        );
    }
    for tool_id in ["bedtools", "mosdepth", "samtools"] {
        assert!(
            !rows.iter().any(|row| { row.starts_with(&format!("bam\tbam.coverage\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for bam.coverage / {tool_id}"
        );
    }
    for tool_id in ["bamtools", "samtools"] {
        assert!(
            !rows
                .iter()
                .any(|row| { row.starts_with(&format!("bam\tbam.mapq_filter\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for bam.mapq_filter / {tool_id}"
        );
    }
    for tool_id in ["picard", "samtools"] {
        assert!(
            !rows
                .iter()
                .any(|row| { row.starts_with(&format!("bam\tbam.length_filter\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for bam.length_filter / {tool_id}"
        );
    }
    for tool_id in ["picard", "samtools"] {
        assert!(
            !rows.iter().any(|row| {
                row.starts_with(&format!("bam\tbam.duplication_metrics\t{tool_id}\t"))
            }),
            "TSV must not retain a registry-drift row for bam.duplication_metrics / {tool_id}"
        );
    }
    assert!(
        !rows.iter().any(|row| { row.starts_with("bam\tbam.insert_size\tpicard\t") }),
        "TSV must not retain a registry-drift row for bam.insert_size / picard"
    );
    assert!(
        !rows.iter().any(|row| { row.starts_with("bam\tbam.gc_bias\tpicard\t") }),
        "TSV must not retain a registry-drift row for bam.gc_bias / picard"
    );
    for tool_id in ["picard", "samtools"] {
        assert!(
            !rows.iter().any(|row| { row.starts_with(&format!("bam\tbam.markdup\t{tool_id}\t")) }),
            "TSV must not retain a registry-drift row for bam.markdup / {tool_id}"
        );
    }
    for tool_id in ["bowtie2", "bwa"] {
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
}
