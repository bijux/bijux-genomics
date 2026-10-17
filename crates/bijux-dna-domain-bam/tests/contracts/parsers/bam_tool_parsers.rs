use std::path::PathBuf;

use bijux_dna_domain_bam::metrics::SexConfidenceClass;
use bijux_dna_domain_bam::metrics::{
    parse_contamination_json, parse_damageprofiler_json, parse_mapdamage2_misincorporation,
    parse_mosdepth_summary, parse_picard_gc_bias_metrics, parse_picard_insert_size_metrics,
    parse_preseq_estimates, parse_pydamage_json, parse_samtools_depth, parse_samtools_flagstat,
    parse_samtools_idxstats, parse_samtools_stats, parse_sex_json,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bam")
        .join("default")
        .join(name)
}

#[test]
fn parse_bam_fixtures_roundtrip() -> anyhow::Result<()> {
    let flagstat = parse_samtools_flagstat(&fixture_path("flagstat.txt"))?;
    assert_eq!(flagstat.total, 1000);
    assert_eq!(flagstat.mapped, 700);

    let (fragment, mapq) = parse_samtools_stats(&fixture_path("samtools_stats.txt"))?;
    assert!(fragment.mean > 0.0);
    assert!(!mapq.histogram.is_empty());

    let idxstats = parse_samtools_idxstats(&fixture_path("idxstats.txt"))?;
    assert_eq!(idxstats.total_mapped, 700);
    assert!(idxstats.reference_mismatch);

    let coverage = parse_mosdepth_summary(&fixture_path("mosdepth.summary.txt"))?;
    assert!(coverage.mean > 0.0);
    assert!(coverage.breadth_1x > 0.0);

    let depth = parse_samtools_depth(&fixture_path("depth.txt"))?;
    assert!(depth.mean > 0.0);

    let complexity = parse_preseq_estimates(&fixture_path("preseq.txt"))?;
    assert_eq!(complexity.projected_reads.len(), 2);

    let pydamage = parse_pydamage_json(&fixture_path("pydamage.json"))?;
    assert!(pydamage.c_to_t_5p > 0.0);

    let damageprofiler = parse_damageprofiler_json(&fixture_path("damageprofiler.json"))?;
    assert!(damageprofiler.c_to_t_5p > 0.0);

    let mapdamage2 = parse_mapdamage2_misincorporation(&fixture_path("mapdamage2.txt"))?;
    assert!(mapdamage2.c_to_t_5p > 0.0);

    let contamination = parse_contamination_json(&fixture_path("contamination.json"))?;
    assert!(contamination.estimate > 0.0);

    let sex = parse_sex_json(&fixture_path("sex.json"))?;
    assert!(sex.sufficient_data);

    let insert = parse_picard_insert_size_metrics(&fixture_path("insert_size.metrics.txt"))?;
    assert!(insert.mean_insert_size > 0.0);
    assert!(insert.read_pairs > 0);

    let gc_bias = parse_picard_gc_bias_metrics(&fixture_path("gc_bias.metrics.txt"))?;
    assert!(gc_bias.total_clusters >= gc_bias.aligned_reads);

    Ok(())
}

#[test]
fn parse_sex_json_honors_explicit_r_y_classification() -> anyhow::Result<()> {
    let sex = parse_sex_json(&fixture_path("rxy_female.json"))?;
    assert_eq!(sex.classification, SexConfidenceClass::Female);
    assert!(sex.sufficient_data);
    assert_eq!(sex.x_to_y_ratio.to_bits(), 0.0_f64.to_bits());
    Ok(())
}

#[test]
fn acquired_downstream_json_is_independent_of_later_path_observations() -> anyhow::Result<()> {
    use bijux_dna_domain_bam::metrics::downstream::{
        contamination::parse_contamination_json_contents, sex::parse_sex_json_contents,
    };
    let owner = bijux_dna_infra::temp_dir("bam-acquired-json-")?;
    let path = owner.path().join("report.json");
    let raw = r#"{"method":"rxy","estimate":0.12,"x_to_y_ratio":0.4,"confidence":0.8}"#;
    std::fs::write(&path, raw)?;
    let first_contamination = serde_json::to_value(parse_contamination_json(&path)?)?;
    let first_sex = serde_json::to_value(parse_sex_json(&path)?)?;
    let acquired = std::fs::read_to_string(&path)?;
    std::fs::write(
        &path,
        r#"{"method":"replacement","estimate":0.8,"x_to_y_ratio":2.0,"confidence":0.9}"#,
    )?;
    assert_eq!(
        serde_json::to_value(parse_contamination_json_contents(&acquired)?)?,
        first_contamination
    );
    assert_eq!(serde_json::to_value(parse_sex_json_contents(&acquired)?)?, first_sex);
    assert_ne!(serde_json::to_value(parse_contamination_json(&path)?)?, first_contamination);
    assert_ne!(serde_json::to_value(parse_sex_json(&path)?)?, first_sex);
    std::fs::remove_file(&path)?;
    assert_eq!(
        serde_json::to_value(parse_contamination_json_contents(&acquired)?)?,
        first_contamination
    );
    assert_eq!(serde_json::to_value(parse_sex_json_contents(&acquired)?)?, first_sex);
    assert!(parse_contamination_json(&path).is_err());
    assert!(parse_sex_json(&path).is_err());
    assert!(parse_contamination_json_contents("{").is_err());
    assert!(parse_sex_json_contents("{").is_err());
    Ok(())
}

#[test]
fn contamination_contents_preserve_defaults_and_filtered_assumptions() -> anyhow::Result<()> {
    use bijux_dna_domain_bam::metrics::downstream::contamination::parse_contamination_json_contents;
    for raw in
        ["{}", "[]", "7", r#"{"foreign":true,"method":7,"estimate":"unknown","assumptions":false}"#]
    {
        assert_eq!(
            serde_json::to_value(parse_contamination_json_contents(raw)?)?,
            serde_json::json!({
                "method":"unknown", "estimate":0.0, "ci_low":0.0, "ci_high":0.0, "assumptions":[]
            })
        );
    }
    let observed = parse_contamination_json_contents(
        r#"{"method":"first","method":"last","estimate":0.12,"assumptions":["retained",7,null]}"#,
    )?;
    assert_eq!(observed.method, "last");
    assert_eq!(observed.assumptions, ["retained"]);
    Ok(())
}

#[test]
fn sex_contents_preserve_explicit_and_legacy_classification() -> anyhow::Result<()> {
    use bijux_dna_domain_bam::metrics::downstream::sex::parse_sex_json_contents;
    for (raw, expected_class, expected_sufficient) in [
        ("{}", SexConfidenceClass::Insufficient, false),
        (r#"{"x_to_y_ratio":0.4,"confidence":0.8}"#, SexConfidenceClass::Male, true),
        (r#"{"x_to_y_ratio":1.6,"confidence":0.8}"#, SexConfidenceClass::Female, true),
        (r#"{"x_to_y_ratio":1.0,"confidence":0.8}"#, SexConfidenceClass::Ambiguous, true),
        (
            r#"{"x_to_y_ratio":0.4,"confidence":0.8,"sufficient_data":false}"#,
            SexConfidenceClass::Insufficient,
            false,
        ),
        (
            r#"{"x_to_y_ratio":0.0,"confidence":0.0,"classification":"female","sufficient_data":true}"#,
            SexConfidenceClass::Female,
            true,
        ),
        (
            r#"{"x_to_y_ratio":0.4,"confidence":0.8,"classification":"foreign"}"#,
            SexConfidenceClass::Male,
            true,
        ),
        (
            r#"{"x_to_y_ratio":0.4,"confidence":0.8,"classification":"ambiguous","sufficient_data":false}"#,
            SexConfidenceClass::Ambiguous,
            false,
        ),
    ] {
        let observed = parse_sex_json_contents(raw)?;
        assert_eq!(observed.classification, expected_class);
        assert_eq!(observed.sufficient_data, expected_sufficient);
    }
    Ok(())
}
