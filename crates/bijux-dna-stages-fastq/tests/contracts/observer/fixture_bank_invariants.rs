#![allow(clippy::expect_used)]

use anyhow::Result;
use bijux_dna_stages_fastq::observer::{
    parse_deduplicate_report, parse_fastqvalidator_count, parse_low_complexity_report,
    parse_seqkit_stats,
};
use std::path::PathBuf;

fn bank_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stage_output_bank/default")
}

#[test]
fn stage_output_bank_has_all_fastq_stage_files() {
    let dir = bank_dir();
    let expected = [
        "fastq.trim_reads.fastp.txt",
        "fastq.validate_reads.seqkit.tsv",
        "fastq.filter_reads.prinseq.txt",
        "fastq.merge_pairs.flash2.txt",
        "fastq.remove_duplicates.clumpify.txt",
        "fastq.remove_duplicates.fastuniq.txt",
        "fastq.filter_low_complexity.bbduk.txt",
        "fastq.filter_low_complexity.dustmasker.txt",
        "fastq.filter_low_complexity.prinseq.txt",
        "fastq.correct_errors.rcorrector.txt",
        "fastq.report_qc.multiqc.txt",
        "fastq.extract_umis.umi_tools.txt",
        "fastq.profile_reads.seqkit.txt",
        "fastq.screen_taxonomy.fastq_screen.tsv",
    ];
    for file in expected {
        let path = dir.join(file);
        assert!(path.exists(), "missing fixture file: {}", path.display());
        let raw = std::fs::read_to_string(&path).expect("read fixture");
        assert!(!raw.trim().is_empty(), "fixture is empty: {}", path.display());
    }
}

#[test]
fn deduplicate_fixture_invariants_parse_metrics() -> Result<()> {
    let fixtures = [
        include_str!(
            "../../fixtures/stage_output_bank/default/fastq.remove_duplicates.fastuniq.txt"
        ),
        include_str!(
            "../../fixtures/stage_output_bank/default/fastq.remove_duplicates.clumpify.txt"
        ),
    ];
    for raw in fixtures {
        let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
        assert!(reads_out <= reads_in);
        let reads_in = u32::try_from(reads_in)
            .unwrap_or_else(|err| panic!("fixture reads_in must fit in u32: {err}"));
        let reads_out = u32::try_from(reads_out)
            .unwrap_or_else(|err| panic!("fixture reads_out must fit in u32: {err}"));
        let retained = f64::from(reads_out) / f64::from(reads_in);
        assert!((0.0..=1.0).contains(&retained));
    }
    Ok(())
}

#[test]
fn low_complexity_fixture_invariants_parse_metrics() -> Result<()> {
    let fixtures = [
        include_str!(
            "../../fixtures/stage_output_bank/default/fastq.filter_low_complexity.bbduk.txt"
        ),
        include_str!(
            "../../fixtures/stage_output_bank/default/fastq.filter_low_complexity.dustmasker.txt"
        ),
        include_str!(
            "../../fixtures/stage_output_bank/default/fastq.filter_low_complexity.prinseq.txt"
        ),
    ];
    for raw in fixtures {
        let removed = parse_low_complexity_report(raw)?;
        let reads_in = kv_u64(raw, "reads_in").unwrap_or_default();
        let reads_out = kv_u64(raw, "reads_out").unwrap_or_default();
        assert!(removed > 0);
        assert_eq!(removed, reads_in.saturating_sub(reads_out));
    }
    Ok(())
}

fn kv_u64(raw: &str, key: &str) -> Option<u64> {
    raw.lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(k, v)| (k.trim() == key).then(|| v.trim().parse::<u64>().ok()).flatten())
}

#[test]
fn seqkit_fixture_roundtrip_preserves_metrics() -> Result<()> {
    let path = bank_dir().join("fastq.validate_reads.seqkit.tsv");
    let raw = std::fs::read_to_string(path)?;
    let metrics = parse_seqkit_stats(&raw)?;
    let reconstructed = format!(
        "file\tformat\ttype\tnum_seqs\tsum_len\tavg_qual\tGC(%)\nreads.fastq.gz\tFASTQ\tDNA\t{}\t{}\t{}\t{}\n",
        metrics.reads, metrics.bases, metrics.mean_q, metrics.gc_percent
    );
    let reparsed = parse_seqkit_stats(&reconstructed)?;
    assert_eq!(reparsed.reads, metrics.reads);
    assert_eq!(reparsed.bases, metrics.bases);
    Ok(())
}

#[test]
fn fastqvalidator_fixture_invariant_nonzero_reads() -> Result<()> {
    let raw = include_str!("../../fixtures/fastqvalidator/default/fastqvalidator_v1.txt");
    let reads = parse_fastqvalidator_count(raw)?;
    assert!(reads > 0);
    Ok(())
}

#[test]
fn screen_fixture_invariant_contamination_rate_bounds() -> Result<()> {
    let raw = std::fs::read_to_string(bank_dir().join("fastq.screen_taxonomy.fastq_screen.tsv"))?;
    let mut unmapped = None;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 && parts[0].eq_ignore_ascii_case("unmapped") {
            let value = parts[2].trim_end_matches('%').parse::<f64>()?;
            unmapped = Some(value);
        }
    }
    let contamination_rate = (100.0 - unmapped.unwrap_or(100.0)).max(0.0) / 100.0;
    assert!((0.0..=1.0).contains(&contamination_rate));
    Ok(())
}
