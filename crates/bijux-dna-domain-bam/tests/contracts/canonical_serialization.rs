#[test]
fn params_and_metrics_canonical() {
    let params = bijux_dna_domain_bam::params::AlignEffectiveParams {
        aligner: "bwa".to_string(),
        preset: "default".to_string(),
        threads: 4,
        reference: "ref".to_string(),
        reference_digest: "sha256:ref".to_string(),
        rg_policy: bijux_dna_domain_bam::types::ReadGroupPolicy::Preserve,
        read_group: bijux_dna_domain_bam::params::ReadGroupSpec::with_defaults("sample"),
        sensitivity_profile: Some("default".to_string()),
        seed_length: Some(19),
        build_indices: true,
        emit_stats: true,
    };
    let metrics = bijux_dna_domain_bam::metrics::AlignmentCountsV1 {
        total: 10,
        primary: 10,
        mapped: 9,
        proper_pair: 0,
        duplicates: 0,
    };
    let params_json = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&params)
            .unwrap_or_else(|err| panic!("canonical params: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8 params: {err}"));
    let metrics_json = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&metrics)
            .unwrap_or_else(|err| panic!("canonical metrics: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8 metrics: {err}"));
    assert!(params_json.contains("aligner"));
    assert!(metrics_json.contains("mapped"));
}
