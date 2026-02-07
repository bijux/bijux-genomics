use bijux_stages_bam::observer::{parse_samtools_flagstat, parse_samtools_idxstats};

#[test]
fn bam_observer_outputs_are_deterministic() -> anyhow::Result<()> {
    let flagstat = include_str!("fixtures/observer/flagstat.txt");
    let idxstats = include_str!("fixtures/observer/idxstats.txt");
    let temp = bijux_infra::temp_dir("bijux-bam-observer")?;
    let flag_path = temp.path().join("flagstat.txt");
    let idx_path = temp.path().join("idxstats.txt");
    bijux_infra::write_bytes(&flag_path, flagstat)?;
    bijux_infra::write_bytes(&idx_path, idxstats)?;
    let flag = parse_samtools_flagstat(&flag_path).expect("flagstat");
    let idx = parse_samtools_idxstats(&idx_path).expect("idxstats");
    let payload = serde_json::json!({"flagstat": flag, "idxstats": idx});
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&payload)
            .expect("canonical"),
    )
    .expect("utf8");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/observer_snapshot.json");
    if std::env::var("UPDATE_CONTRACTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, &actual).expect("write snapshot");
    }
    let expected = std::fs::read_to_string(&path).expect("read snapshot");
    assert_eq!(actual, expected);
    Ok(())
}
