use bijux_stages_bam::observer::{parse_samtools_flagstat, parse_samtools_idxstats};

#[test]
fn bam_metrics_have_required_fields() -> anyhow::Result<()> {
    let flagstat = include_str!("fixtures/observer/flagstat.txt");
    let idxstats = include_str!("fixtures/observer/idxstats.txt");
    let temp = bijux_infra::temp_dir("bijux-bam-metrics")?;
    let flag_path = temp.path().join("flagstat.txt");
    let idx_path = temp.path().join("idxstats.txt");
    bijux_infra::write_bytes(&flag_path, flagstat)?;
    bijux_infra::write_bytes(&idx_path, idxstats)?;
    let flag = parse_samtools_flagstat(&flag_path).expect("flagstat");
    assert!(flag.total > 0, "flagstat total missing");
    let idx = parse_samtools_idxstats(&idx_path).expect("idxstats");
    assert!(!idx.contigs.is_empty(), "idxstats contigs missing");
    Ok(())
}
