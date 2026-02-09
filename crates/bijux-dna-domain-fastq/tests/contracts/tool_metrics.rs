use std::path::PathBuf;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_domain_fastq::metrics::{
    BrackenClassificationMetricsV1, FastqScanMetricsV1, KrakenUniqClassificationMetricsV1,
    SeqfuMetricsV1,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tool_metrics/default")
        .join(name)
}

fn load_json(name: &str) -> anyhow::Result<serde_json::Value> {
    let raw = std::fs::read_to_string(fixture(name))?;
    Ok(serde_json::from_str(&raw)?)
}

#[test]
fn parse_fastq_scan_and_seqfu_metrics_with_invariants() -> anyhow::Result<()> {
    let fastq_scan: FastqScanMetricsV1 = serde_json::from_value(load_json("fastq_scan.json")?)?;
    let seqfu: SeqfuMetricsV1 = serde_json::from_value(load_json("seqfu.json")?)?;

    for summary in [&fastq_scan.summary, &seqfu.summary] {
        assert!(summary.reads > 0);
        assert!(summary.bases_bp > 0);
        assert!(summary.mean_read_length_bp > 0.0);
        assert!((0.0..=45.0).contains(&summary.qscore.mean_phred));
        assert!((0.0..=45.0).contains(&summary.qscore.median_phred));
        assert!((0.0..=45.0).contains(&summary.qscore.p10_phred));
        assert!((0.0..=45.0).contains(&summary.qscore.p90_phred));
        if let Some(dup_pct) = summary.duplication_estimate_pct {
            assert!((0.0..=100.0).contains(&dup_pct));
        }
    }

    let canonical = to_canonical_json_bytes(&fastq_scan)?;
    let reparsed: FastqScanMetricsV1 = serde_json::from_slice(&canonical)?;
    assert_eq!(reparsed.summary.reads, fastq_scan.summary.reads);
    Ok(())
}

#[test]
fn parse_krakenuniq_and_bracken_metrics_with_invariants() -> anyhow::Result<()> {
    let krakenuniq: KrakenUniqClassificationMetricsV1 =
        serde_json::from_value(load_json("krakenuniq.json")?)?;
    let bracken: BrackenClassificationMetricsV1 =
        serde_json::from_value(load_json("bracken.json")?)?;

    assert!(!krakenuniq.provenance.db_name.trim().is_empty());
    assert!(!krakenuniq.provenance.db_version.trim().is_empty());
    assert!(!krakenuniq.provenance.db_hash.trim().is_empty());

    let mut kraken_fraction_sum = 0.0_f64;
    for row in &krakenuniq.taxonomy_table {
        assert!(row.unique_kmer_count > 0);
        if let Some(conf) = row.confidence {
            assert!((0.0..=1.0).contains(&conf));
        }
        if let Some(frac) = row.taxonomy.fraction {
            assert!((0.0..=1.0).contains(&frac));
            kraken_fraction_sum += frac;
        }
    }
    assert!(kraken_fraction_sum <= 1.0 + 1e-9);

    let mut bracken_fraction_sum = 0.0_f64;
    for row in &bracken.taxonomy_table {
        assert!(row.estimated_reads >= 0.0);
        if let Some(frac) = row.estimated_fraction {
            assert!((0.0..=1.0).contains(&frac));
            bracken_fraction_sum += frac;
        }
    }
    assert!(bracken_fraction_sum <= 1.0 + 1e-9);

    let canonical = to_canonical_json_bytes(&bracken)?;
    let reparsed: BrackenClassificationMetricsV1 = serde_json::from_slice(&canonical)?;
    assert_eq!(reparsed.taxonomy_table.len(), bracken.taxonomy_table.len());
    Ok(())
}
