use std::path::PathBuf;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_domain_bam::metrics::{
    AdDeamMetricsV1, ContamMixMetricsV1, NgsBriggsMetricsV1, PmdtoolsMetricsV1, SchmutziMetricsV1,
    VerifyBamId2MetricsV1,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tool_metrics/default")
        .join(name)
}

fn load<T: serde::de::DeserializeOwned>(name: &str) -> anyhow::Result<T> {
    let raw = std::fs::read_to_string(fixture(name))?;
    Ok(serde_json::from_str(&raw)?)
}

fn assert_damage_core(core: &bijux_dna_domain_bam::metrics::DamageCoreFieldsV1) {
    assert!((0.0..=1.0).contains(&core.c_to_t_5p));
    assert!((0.0..=1.0).contains(&core.g_to_a_3p));
    assert!(core.reads_considered > 0);
}

fn assert_pmd_distribution(dist: &bijux_dna_domain_bam::metrics::PmdScoreDistributionV1) {
    assert!(dist.threshold >= 0.0);
    let mut last_upper = f64::NEG_INFINITY;
    for bin in &dist.bins {
        assert!(bin.lower_bound <= bin.upper_bound);
        assert!(bin.lower_bound >= last_upper);
        last_upper = bin.upper_bound;
    }
}

fn assert_contamination_bounds(tool: &bijux_dna_domain_bam::metrics::ContaminationToolMetricsV1) {
    assert!((0.0..=1.0).contains(&tool.estimate));
    assert!((0.0..=1.0).contains(&tool.ci_low));
    assert!((0.0..=1.0).contains(&tool.ci_high));
    assert!(tool.ci_low <= tool.estimate && tool.estimate <= tool.ci_high);
    assert!(!tool.required_inputs.reference_panel.trim().is_empty());
    for warning in &tool.warnings {
        assert!(!warning.code.trim().is_empty());
    }
}

#[test]
fn parse_damage_tool_metrics_with_invariants() -> anyhow::Result<()> {
    let pmdtools: PmdtoolsMetricsV1 = load("pmdtools.json")?;
    let ngsbriggs: NgsBriggsMetricsV1 = load("ngsbriggs.json")?;
    let addeam: AdDeamMetricsV1 = load("addeam.json")?;

    assert_damage_core(&pmdtools.core);
    assert_damage_core(&ngsbriggs.core);
    assert_damage_core(&addeam.core);
    assert_pmd_distribution(&pmdtools.pmd_distribution);
    assert_pmd_distribution(&ngsbriggs.pmd_distribution);
    assert_pmd_distribution(&addeam.pmd_distribution);
    assert!(ngsbriggs.lambda >= 0.0);
    assert!(ngsbriggs.delta_s >= 0.0);
    assert!(addeam.cluster_count > 0);

    let canonical = to_canonical_json_bytes(&ngsbriggs)?;
    let reparsed: NgsBriggsMetricsV1 = serde_json::from_slice(&canonical)?;
    assert_eq!(reparsed.core.tool, ngsbriggs.core.tool);
    Ok(())
}

#[test]
fn parse_contamination_tool_metrics_with_invariants() -> anyhow::Result<()> {
    let schmutzi: SchmutziMetricsV1 = load("schmutzi.json")?;
    let verifybamid2: VerifyBamId2MetricsV1 = load("verifybamid2.json")?;
    let contammix: ContamMixMetricsV1 = load("contammix.json")?;

    assert_contamination_bounds(&schmutzi.contamination);
    assert_contamination_bounds(&verifybamid2.contamination);
    assert_contamination_bounds(&contammix.contamination);

    let canonical = to_canonical_json_bytes(&schmutzi)?;
    let reparsed: SchmutziMetricsV1 = serde_json::from_slice(&canonical)?;
    assert_eq!(reparsed.contamination.tool, "schmutzi");
    Ok(())
}
