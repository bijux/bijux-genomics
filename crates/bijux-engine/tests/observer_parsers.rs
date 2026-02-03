use anyhow::Result;
use bijux_engine::primitives::parse_fastqvalidator_count;

#[test]
fn parse_fastqvalidator_count_parses_total_reads() -> Result<()> {
    let stdout = "Some header\nTotal Reads: 12345\nDone\n";
    let count = parse_fastqvalidator_count(stdout)?;
    assert_eq!(count, 12345);
    Ok(())
}
