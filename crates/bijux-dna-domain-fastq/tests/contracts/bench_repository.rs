use bijux_dna_domain_fastq::BenchQueryContext;

#[test]
fn bench_query_context_tracks_provenance_axes() {
    let context = BenchQueryContext::new()
        .with_params_hash("params-hash")
        .with_image_digest("sha256:image")
        .with_stage_contract_hash("contract-hash")
        .with_reference_hash("reference-hash")
        .with_database_hash("database-hash")
        .with_bank_hash("adapter_bank", "adapter-bank-hash")
        .with_lineage_hash("lineage-hash");

    assert_eq!(context.params_hash.as_deref(), Some("params-hash"));
    assert_eq!(context.image_digest.as_deref(), Some("sha256:image"));
    assert_eq!(
        context.stage_contract_hash.as_deref(),
        Some("contract-hash")
    );
    assert_eq!(context.reference_hash.as_deref(), Some("reference-hash"));
    assert_eq!(context.database_hash.as_deref(), Some("database-hash"));
    assert_eq!(
        context.bank_hashes.get("adapter_bank").map(String::as_str),
        Some("adapter-bank-hash")
    );
    assert_eq!(context.lineage_hash.as_deref(), Some("lineage-hash"));
    assert!(!context.is_empty());
}

#[test]
fn empty_bench_query_context_reports_no_filters() {
    let context = BenchQueryContext::new();

    assert!(context.is_empty());
}
