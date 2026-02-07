#[test]
fn params_and_metrics_canonical() {
    let params = bijux_domain_bam::params::AlignEffectiveParams {
        aligner: "bwa".to_string(),
        preset: "default".to_string(),
        threads: 4,
        reference: "ref".to_string(),
        reference_digest: "sha256:ref".to_string(),
        rg_policy: bijux_domain_bam::types::ReadGroupPolicy::Preserve,
        read_group: bijux_domain_bam::params::ReadGroupSpec::with_defaults("sample"),
        build_indices: true,
        emit_stats: true,
    };
    let metrics = bijux_domain_bam::metrics::AlignmentCountsV1 {
        total: 10,
        primary: 10,
        mapped: 9,
        proper_pair: 0,
        duplicates: 0,
    };
    let params_json = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&params).expect("canonical"),
    )
    .expect("utf8");
    let metrics_json = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&metrics).expect("canonical"),
    )
    .expect("utf8");
    assert!(params_json.contains("min_quality"));
    assert!(metrics_json.contains("mapped"));
}
