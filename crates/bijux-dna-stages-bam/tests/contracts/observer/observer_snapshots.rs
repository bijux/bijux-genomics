use std::path::PathBuf;

use bijux_dna_stages_bam::observer::{
    parse_contamination_json, parse_damageprofiler_json, parse_mapdamage2_misincorporation,
    parse_mosdepth_summary, parse_picard_gc_bias_metrics, parse_picard_insert_size_metrics,
    parse_preseq_estimates, parse_pydamage_json, parse_samtools_depth, parse_samtools_flagstat,
    parse_samtools_idxstats, parse_samtools_stats, parse_sex_json,
};

fn fixture(path: &str) -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/observer/default")
        .join(path)
}

fn snapshot_path(name: &str) -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/observer_snapshots/default")
        .join(format!("{name}.json"))
}

fn write_snapshot(name: &str, payload: &serde_json::Value) {
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(payload)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    let path = snapshot_path(name);
    if std::env::var("UPDATE_CONTRACTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, &actual).unwrap_or_else(|err| panic!("write snapshot: {err}"));
    }
    let expected =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read snapshot: {err}"));
    assert_eq!(actual, expected, "snapshot mismatch for {name}");
}

#[test]
fn snapshot_flagstat() -> anyhow::Result<()> {
    let flagstat = parse_samtools_flagstat(&fixture("flagstat.txt"))?;
    write_snapshot("flagstat", &serde_json::to_value(flagstat)?);
    Ok(())
}

#[test]
fn snapshot_idxstats() -> anyhow::Result<()> {
    let idxstats = parse_samtools_idxstats(&fixture("idxstats.txt"))?;
    write_snapshot("idxstats", &serde_json::to_value(idxstats)?);
    Ok(())
}

#[test]
fn snapshot_stats() -> anyhow::Result<()> {
    let (fragment, mapq) = parse_samtools_stats(&fixture("stats.txt"))?;
    write_snapshot(
        "samtools_stats",
        &serde_json::json!({"fragment": fragment, "mapq": mapq}),
    );
    Ok(())
}

#[test]
fn snapshot_preseq() -> anyhow::Result<()> {
    let preseq = parse_preseq_estimates(&fixture("preseq.txt"))?;
    write_snapshot("preseq", &serde_json::to_value(preseq)?);
    Ok(())
}

#[test]
fn snapshot_mosdepth() -> anyhow::Result<()> {
    let mos = parse_mosdepth_summary(&fixture("mosdepth.summary.txt"))?;
    write_snapshot("mosdepth", &serde_json::to_value(mos)?);
    Ok(())
}

#[test]
fn snapshot_mapdamage2() -> anyhow::Result<()> {
    let mapdamage = parse_mapdamage2_misincorporation(&fixture("mapdamage2.txt"))?;
    write_snapshot("mapdamage2", &serde_json::to_value(mapdamage)?);
    Ok(())
}

#[test]
fn snapshot_pydamage() -> anyhow::Result<()> {
    let pydamage = parse_pydamage_json(&fixture("pydamage.json"))?;
    write_snapshot("pydamage", &serde_json::to_value(pydamage)?);
    Ok(())
}

#[test]
fn snapshot_damageprofiler() -> anyhow::Result<()> {
    let damage = parse_damageprofiler_json(&fixture("damageprofiler.json"))?;
    write_snapshot("damageprofiler", &serde_json::to_value(damage)?);
    Ok(())
}

#[test]
fn snapshot_contamination() -> anyhow::Result<()> {
    let contamination = parse_contamination_json(&fixture("contamination.json"))?;
    write_snapshot("contamination", &serde_json::to_value(contamination)?);
    Ok(())
}

#[test]
fn snapshot_sex() -> anyhow::Result<()> {
    let sex = parse_sex_json(&fixture("sex.json"))?;
    write_snapshot("sex", &serde_json::to_value(sex)?);
    Ok(())
}

#[test]
fn snapshot_depth() -> anyhow::Result<()> {
    let depth = parse_samtools_depth(&fixture("samtools.depth.txt"))?;
    write_snapshot("samtools_depth", &serde_json::to_value(depth)?);
    Ok(())
}

#[test]
fn snapshot_insert_size() -> anyhow::Result<()> {
    let insert = parse_picard_insert_size_metrics(&fixture("insert_size.metrics.txt"))?;
    write_snapshot("insert_size", &serde_json::to_value(insert)?);
    Ok(())
}

#[test]
fn snapshot_gc_bias() -> anyhow::Result<()> {
    let gc_bias = parse_picard_gc_bias_metrics(&fixture("gc_bias.metrics.txt"))?;
    write_snapshot("gc_bias", &serde_json::to_value(gc_bias)?);
    Ok(())
}
