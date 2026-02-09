#[test]
fn trim_params_canonical_serialization() {
    let params = bijux_dna_domain_fastq::params::trim::TrimEffectiveParams {
        paired_mode: bijux_dna_domain_fastq::params::PairedMode::SingleEnd,
        threads: 4,
        min_len: 30,
        q_cutoff: Some(20),
        adapter_policy: "auto".to_string(),
        damage_mode: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
    };
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&params)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("trim_params")
        .join("default")
        .join("trim_params.json");
    if std::env::var("UPDATE_CONTRACTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, &actual)
            .unwrap_or_else(|err| panic!("write snapshot {}: {err}", path.display()));
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read snapshot {}: {err}", path.display()));
    assert_eq!(actual, expected);
}
