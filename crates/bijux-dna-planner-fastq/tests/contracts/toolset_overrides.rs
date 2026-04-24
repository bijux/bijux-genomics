use std::collections::BTreeMap;

use bijux_dna_planner_fastq::apply_toolset_overrides;

#[test]
fn toolset_override_precedence_is_stable() {
    let mut base = BTreeMap::new();
    base.insert("fastq.trim_reads".to_string(), vec!["fastp".to_string(), "cutadapt".to_string()]);
    let mut profile = BTreeMap::new();
    profile
        .insert("fastq.trim_reads".to_string(), vec!["bbduk".to_string(), "cutadapt".to_string()]);
    let mut cli = BTreeMap::new();
    cli.insert(
        "fastq.trim_reads".to_string(),
        vec!["trimmomatic".to_string(), "fastp".to_string()],
    );
    let mut forced = BTreeMap::new();
    forced.insert(
        "fastq.trim_reads".to_string(),
        vec!["SeqPurge".to_string(), "fastp".to_string(), "seqpurge".to_string()],
    );

    let merged = apply_toolset_overrides(base, profile, cli, forced);
    assert_eq!(
        merged.get("fastq.trim_reads"),
        Some(&vec!["fastp".to_string(), "seqpurge".to_string()])
    );
}

#[test]
fn toolset_override_merge_keeps_stage_boundaries_independent() {
    let mut base = BTreeMap::new();
    base.insert("fastq.trim_reads".to_string(), vec!["fastp".to_string(), "cutadapt".to_string()]);
    base.insert("fastq.screen_taxonomy".to_string(), vec!["kraken2".to_string()]);

    let merged = apply_toolset_overrides(base, BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    assert_eq!(
        merged.get("fastq.trim_reads"),
        Some(&vec!["cutadapt".to_string(), "fastp".to_string()])
    );
    assert_eq!(merged.get("fastq.screen_taxonomy"), Some(&vec!["kraken2".to_string()]));
}
