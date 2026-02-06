use std::path::Path;

use bijux_stages_bam::observer::{
    parse_contamination_json, parse_damageprofiler_json, parse_mapdamage2_misincorporation,
    parse_mosdepth_summary, parse_preseq_estimates, parse_pydamage_json, parse_samtools_depth,
    parse_samtools_flagstat, parse_samtools_idxstats, parse_samtools_stats, parse_sex_json,
};

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new("tests/fixtures/observer").join(path)
}

#[test]
fn parses_alignment_and_quality_observers() -> anyhow::Result<()> {
    let flagstat = parse_samtools_flagstat(&fixture("flagstat.txt"))?;
    assert_eq!(flagstat.total, 10);
    let (_frag, _mapq) = parse_samtools_stats(&fixture("stats.txt"))?;
    let idx = parse_samtools_idxstats(&fixture("idxstats.txt"))?;
    assert_eq!(idx.total_mapped, 5);
    Ok(())
}

#[test]
fn parses_complexity_and_coverage_observers() -> anyhow::Result<()> {
    let complexity = parse_preseq_estimates(&fixture("preseq.txt"))?;
    assert_eq!(complexity.observed_reads, 80);
    let mos = parse_mosdepth_summary(&fixture("mosdepth.summary.txt"))?;
    assert!(mos.mean > 0.0);
    let depth = parse_samtools_depth(&fixture("samtools.depth.txt"))?;
    assert!(depth.mean >= 0.0);
    Ok(())
}

#[test]
fn parses_damage_and_contamination_observers() -> anyhow::Result<()> {
    let pydamage = parse_pydamage_json(&fixture("pydamage.json"))?;
    assert!(pydamage.c_to_t_5p > 0.0);
    let damageprofiler = parse_damageprofiler_json(&fixture("damageprofiler.json"))?;
    assert!(damageprofiler.g_to_a_3p > 0.0);
    let mapdamage = parse_mapdamage2_misincorporation(&fixture("mapdamage2.txt"))?;
    assert!(mapdamage.c_to_t_5p > 0.0);
    let contamination = parse_contamination_json(&fixture("contamination.json"))?;
    assert!(contamination.estimate > 0.0);
    Ok(())
}

#[test]
fn parses_sex_observer() -> anyhow::Result<()> {
    let sex = parse_sex_json(&fixture("sex.json"))?;
    assert!(sex.confidence > 0.0);
    Ok(())
}
