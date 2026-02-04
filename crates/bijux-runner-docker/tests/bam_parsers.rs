use std::path::PathBuf;

use bijux_runner_docker::primitives::{
    parse_contamination_json, parse_damageprofiler_json, parse_mosdepth_summary,
    parse_preseq_estimates, parse_pydamage_json, parse_samtools_flagstat, parse_samtools_stats,
    parse_sex_json,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bam")
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

    let coverage = parse_mosdepth_summary(&fixture_path("mosdepth.summary.txt"))?;
    assert!(coverage.mean > 0.0);

    let complexity = parse_preseq_estimates(&fixture_path("preseq.txt"))?;
    assert_eq!(complexity.projected_reads.len(), 2);

    let pydamage = parse_pydamage_json(&fixture_path("pydamage.json"))?;
    assert!(pydamage.c_to_t_5p > 0.0);

    let damageprofiler = parse_damageprofiler_json(&fixture_path("damageprofiler.json"))?;
    assert!(damageprofiler.c_to_t_5p > 0.0);

    let contamination = parse_contamination_json(&fixture_path("contamination.json"))?;
    assert!(contamination.estimate > 0.0);

    let sex = parse_sex_json(&fixture_path("sex.json"))?;
    assert!(sex.sufficient_data);

    Ok(())
}
