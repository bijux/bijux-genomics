use std::path::PathBuf;

use bijux_stages_bam::observer::{
    parse_contamination_json, parse_damageprofiler_json, parse_mapdamage2_misincorporation,
    parse_pydamage_json,
};

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("observer")
        .join(path)
}

#[test]
fn contamination_metrics_are_complete() {
    let metrics =
        parse_contamination_json(&fixture("contamination.json")).expect("parse contamination json");
    assert_ne!(metrics.method, "unknown");
    assert!(metrics.estimate > 0.0);
    assert!(metrics.ci_high >= metrics.ci_low);
    assert!(!metrics.assumptions.is_empty());
}

#[test]
fn damage_metrics_include_required_fields() {
    let pydamage = parse_pydamage_json(&fixture("pydamage.json")).expect("parse pydamage");
    assert!(pydamage.c_to_t_5p > 0.0);
    assert!(pydamage.g_to_a_3p > 0.0);

    let mapdamage =
        parse_mapdamage2_misincorporation(&fixture("mapdamage2.txt")).expect("mapdamage2");
    assert!(mapdamage.c_to_t_5p > 0.0);
    assert!(mapdamage.g_to_a_3p > 0.0);

    let damageprofiler =
        parse_damageprofiler_json(&fixture("damageprofiler.json")).expect("damageprofiler");
    assert!(damageprofiler.c_to_t_5p > 0.0);
    assert!(damageprofiler.g_to_a_3p > 0.0);
}
