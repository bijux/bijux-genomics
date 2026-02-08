use anyhow::Result;
use bijux_stages_fastq::metrics::filters::parse_screen_report;

#[test]
fn parse_screen_report_parses_fixture() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("screen.tsv");
    let raw = include_str!("../fixtures/screen/default/screen_report_v1.tsv");
    std::fs::write(&path, raw)?;
    let (unmapped, summary) = parse_screen_report(&path)?;
    assert!(unmapped >= 0.0);
    assert!(summary.get("entries").is_some());
    Ok(())
}
