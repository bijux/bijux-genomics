#![allow(clippy::expect_used)]

use bijux_dna_stages_fastq::observer::parse_seqkit_stats;

#[test]
fn seqkit_stats_deterministic() {
    let stdout = include_str!("../../fixtures/seqkit/default/seqkit_stats_v1.txt");
    let metrics = parse_seqkit_stats(stdout).expect("parse");
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&metrics).expect("canonical"),
    )
    .expect("utf8");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("seqkit_stats")
        .join("default")
        .join("seqkit_stats.json");
    if std::env::var("UPDATE_CONTRACTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, &actual).expect("write snapshot");
    }
    let expected = std::fs::read_to_string(&path).expect("read snapshot");
    assert_eq!(actual, expected);
}
