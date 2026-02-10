use std::path::PathBuf;

use bijux_dna_stages_bam::observer::{
    parse_contamination_json, parse_damageprofiler_json, parse_mapdamage2_misincorporation,
    parse_picard_gc_bias_metrics, parse_picard_insert_size_metrics, parse_pydamage_json,
};

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("observer")
        .join("default")
        .join(path)
}

#[test]
fn contamination_metrics_are_complete() {
    let metrics = parse_contamination_json(&fixture("contamination.json"))
        .unwrap_or_else(|err| panic!("parse contamination json: {err}"));
    assert_ne!(metrics.method, "unknown");
    assert!(metrics.estimate > 0.0);
    assert!(metrics.ci_high >= metrics.ci_low);
    assert!(!metrics.assumptions.is_empty());
}

#[test]
fn damage_metrics_include_required_fields() {
    let pydamage = parse_pydamage_json(&fixture("pydamage.json"))
        .unwrap_or_else(|err| panic!("parse pydamage: {err}"));
    assert!(pydamage.c_to_t_5p > 0.0);
    assert!(pydamage.g_to_a_3p > 0.0);

    let mapdamage = parse_mapdamage2_misincorporation(&fixture("mapdamage2.txt"))
        .unwrap_or_else(|err| panic!("mapdamage2: {err}"));
    assert!(mapdamage.c_to_t_5p > 0.0);
    assert!(mapdamage.g_to_a_3p > 0.0);

    let damageprofiler = parse_damageprofiler_json(&fixture("damageprofiler.json"))
        .unwrap_or_else(|err| panic!("damageprofiler: {err}"));
    assert!(damageprofiler.c_to_t_5p > 0.0);
    assert!(damageprofiler.g_to_a_3p > 0.0);
}

#[test]
fn insert_size_and_gc_bias_metrics_are_complete() {
    let insert = parse_picard_insert_size_metrics(&fixture("insert_size.metrics.txt"))
        .unwrap_or_else(|err| panic!("parse insert-size metrics: {err}"));
    assert!(insert.mean_insert_size > 0.0);
    assert!(insert.max_insert_size >= insert.min_insert_size);
    assert!(insert.read_pairs > 0);
    assert!((0.0..=1.0).contains(&insert.pair_orientation_fr_fraction));

    let gc_bias = parse_picard_gc_bias_metrics(&fixture("gc_bias.metrics.txt"))
        .unwrap_or_else(|err| panic!("parse gc-bias metrics: {err}"));
    assert!(gc_bias.total_clusters >= gc_bias.aligned_reads);
    assert!(gc_bias.windows > 0);
    assert!(gc_bias.at_dropout >= 0.0);
    assert!(gc_bias.gc_dropout >= 0.0);
}
