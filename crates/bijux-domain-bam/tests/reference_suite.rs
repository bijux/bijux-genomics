use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_core::prelude::invariants::InvariantStatusV1;
use bijux_domain_bam::metrics::{
    authenticity_score, evaluate_bam_invariants, BamInvariantThresholds, BamMetricsV1,
    SexConfidenceClass,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("reference")
        .join(name)
}

fn load_metrics(name: &str) -> Result<BamMetricsV1> {
    let raw = fs::read_to_string(fixture(name))?;
    Ok(serde_json::from_str(&raw)?)
}

#[test]
fn reference_authentic_adna_scores_high() -> Result<()> {
    let metrics = load_metrics("authentic_adna.json")?;
    let score = authenticity_score(&metrics);
    assert!(score.score >= 0.5);
    Ok(())
}

#[test]
fn reference_modern_contaminated_flags_issue() -> Result<()> {
    let metrics = load_metrics("modern_contaminated.json")?;
    let thresholds = BamInvariantThresholds::default();
    let eval = evaluate_bam_invariants("bam.contamination", &metrics, &thresholds);
    assert!(eval
        .results
        .iter()
        .any(|r| matches!(r.status, InvariantStatusV1::Fail)));
    Ok(())
}

#[test]
fn reference_low_complexity_warns() -> Result<()> {
    let metrics = load_metrics("low_complexity.json")?;
    let thresholds = BamInvariantThresholds::default();
    let eval = evaluate_bam_invariants("bam.complexity", &metrics, &thresholds);
    assert!(eval
        .results
        .iter()
        .any(|r| r.id == "complexity_vs_duplicates"));
    Ok(())
}

#[test]
fn reference_sex_classification_is_stable() -> Result<()> {
    let metrics = load_metrics("authentic_adna.json")?;
    assert_eq!(metrics.sex.classification, SexConfidenceClass::Male);
    Ok(())
}

#[test]
fn reference_kinship_pair_sufficient() -> Result<()> {
    let metrics_a = load_metrics("kinship_pair_a.json")?;
    let metrics_b = load_metrics("kinship_pair_b.json")?;
    assert!(metrics_a.kinship_sufficiency.sufficient);
    assert!(metrics_b.kinship_sufficiency.sufficient);
    Ok(())
}
