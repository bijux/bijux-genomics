use std::fs;

#[test]
fn fixtures_are_documented() -> anyhow::Result<()> {
    let doc = crate::support::crate_root("bijux-dna-bench")?.join("docs").join("BENCH_FORMAT.md");
    let content = fs::read_to_string(&doc)?;
    assert!(content.contains("decision.json"), "BENCH_FORMAT.md must describe decision.json");
    Ok(())
}
