use anyhow::Result;
use bijux_dna_stages_fastq::observer::{parse_fastqvalidator_count, parse_seqkit_stats};
use std::path::PathBuf;

fn bank_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stage_output_bank/default")
}

#[test]
fn stage_output_bank_has_all_fastq_stage_files() {
    let dir = bank_dir();
    let expected = [
        "fastq.trim.fastp.txt",
        "fastq.validate_pre.seqkit.tsv",
        "fastq.filter.prinseq.txt",
        "fastq.merge.flash2.txt",
        "fastq.correct.rcorrector.txt",
        "fastq.qc_post.multiqc.txt",
        "fastq.umi.umi_tools.txt",
        "fastq.stats_neutral.seqkit.txt",
        "fastq.screen.fastq_screen.tsv",
    ];
    for file in expected {
        let path = dir.join(file);
        assert!(path.exists(), "missing fixture file: {}", path.display());
        let raw = std::fs::read_to_string(&path).expect("read fixture");
        assert!(
            !raw.trim().is_empty(),
            "fixture is empty: {}",
            path.display()
        );
    }
}

#[test]
fn seqkit_fixture_roundtrip_preserves_metrics() -> Result<()> {
    let path = bank_dir().join("fastq.validate_pre.seqkit.tsv");
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
    let raw = std::fs::read_to_string(bank_dir().join("fastq.screen.fastq_screen.tsv"))?;
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
