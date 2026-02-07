#[test]
fn trim_params_canonical_serialization() {
    let params = bijux_domain_fastq::params::TrimEffectiveParams {
        paired_mode: bijux_domain_fastq::params::PairedMode::SingleEnd,
        threads: 4,
        min_len: 30,
        q_cutoff: Some(20),
        adapter_policy: "auto".to_string(),
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
    };
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&params).expect("canonical"),
    )
    .expect("utf8");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/trim_params.json");
    if std::env::var("UPDATE_CONTRACTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, &actual).expect("write snapshot");
    }
    let expected = std::fs::read_to_string(&path).expect("read snapshot");
    assert_eq!(actual, expected);
}
