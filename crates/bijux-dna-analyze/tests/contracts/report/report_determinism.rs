use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_analyze::load::load_facts;
use bijux_dna_analyze::report::write_run_report_from_facts;
use bijux_dna_core::contract::canonical::to_canonical_json_bytes;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("pipelines")
        .join("fastq-to-fastq__default__v1")
}

#[test]
fn report_json_is_deterministic() -> Result<()> {
    let root = fixture_root();
    let facts_path = root.join("facts.jsonl");
    let facts = load_facts(&facts_path).map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let report_a = write_run_report_from_facts(&root, &facts)?;
    let report_b = write_run_report_from_facts(&root, &facts)?;
    let raw_a = std::fs::read_to_string(report_a)?;
    let raw_b = std::fs::read_to_string(report_b)?;
    let json_a: serde_json::Value = serde_json::from_str(&raw_a)?;
    let json_b: serde_json::Value = serde_json::from_str(&raw_b)?;
    let canon_a = to_canonical_json_bytes(&json_a)?;
    let canon_b = to_canonical_json_bytes(&json_b)?;
    assert_eq!(canon_a, canon_b);
    Ok(())
}
