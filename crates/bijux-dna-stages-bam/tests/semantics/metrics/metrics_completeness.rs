use bijux_dna_stages_bam::observer::{parse_samtools_flagstat, parse_samtools_idxstats};

#[test]
fn bam_metrics_have_required_fields() -> anyhow::Result<()> {
    let flagstat = include_str!("../../fixtures/observer/default/flagstat.txt");
    let idxstats = include_str!("../../fixtures/observer/default/idxstats.txt");
    let temp = bijux_dna_infra::temp_dir("bijux-dna-bam-metrics")?;
    let flag_path = temp.path().join("flagstat.txt");
    let idx_path = temp.path().join("idxstats.txt");
    bijux_dna_infra::write_bytes(&flag_path, flagstat)?;
    bijux_dna_infra::write_bytes(&idx_path, idxstats)?;
    let flag = parse_samtools_flagstat(&flag_path).unwrap_or_else(|err| panic!("flagstat: {err}"));
    assert!(flag.total > 0, "flagstat total missing");
    let idx = parse_samtools_idxstats(&idx_path).unwrap_or_else(|err| panic!("idxstats: {err}"));
    assert!(!idx.contigs.is_empty(), "idxstats contigs missing");
    Ok(())
}

#[test]
fn bam_metric_discovery_skips_directories_named_like_metric_files() -> anyhow::Result<()> {
    let flagstat = include_str!("../../fixtures/observer/default/flagstat.txt");
    let temp = bijux_dna_infra::temp_dir("bijux-dna-bam-metrics-discovery")?;
    std::fs::create_dir(temp.path().join("flagstat.after.txt"))?;
    bijux_dna_infra::write_bytes(temp.path().join("flagstat.txt"), flagstat)?;

    let metrics = bijux_dna_stages_bam::metrics::bam_metrics_from_dir(temp.path());

    assert_eq!(metrics.alignment.total, 10);
    Ok(())
}
