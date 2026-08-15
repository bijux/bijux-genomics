use bijux_dna_bench_model::{
    validate_benchmark_data_source_spec, BenchmarkCorpusClass, BenchmarkDataSourceCollection,
    BenchmarkDataSourceSpec, CorpusEvaluationDesign, CorpusSourcePolicy,
    BENCHMARK_DATA_SOURCE_SCHEMA_V1,
};

#[test]
fn generic_source_contract_requires_real_ordered_truth_authorities() {
    let source = source_spec();
    assert!(validate_benchmark_data_source_spec(&source).is_ok());

    let mut fixture = source.clone();
    fixture.policy.class = BenchmarkCorpusClass::TestFixture;
    assert!(validate_benchmark_data_source_spec(&fixture)
        .is_err_and(|error| error.to_string().contains("publishable real data")));

    let mut underfilled = source.clone();
    underfilled.target_sample_count = 11;
    assert!(validate_benchmark_data_source_spec(&underfilled)
        .is_err_and(|error| error.to_string().contains("target sample count")));

    let mut missing_split = source.clone();
    missing_split.evaluation.held_out_split.clear();
    assert!(validate_benchmark_data_source_spec(&missing_split)
        .is_err_and(|error| error.to_string().contains("held-out split")));

    let mut unordered_uses = source.clone();
    unordered_uses.evaluation.allowed_uses.reverse();
    assert!(validate_benchmark_data_source_spec(&unordered_uses)
        .is_err_and(|error| error.to_string().contains("strictly ordered")));

    let mut unknown_type = source;
    unknown_type.collections[0].data_types = vec!["fasta".to_string()];
    assert!(validate_benchmark_data_source_spec(&unknown_type)
        .is_err_and(|error| error.to_string().contains("absent from the corpus")));
}

#[test]
fn explicit_selected_accessions_bind_the_planned_collection_count() {
    let mut source = source_spec();
    source.collections[0].planned_sample_count = Some(2);
    source.collections[0].selected_accessions =
        vec!["SAMPLE-001".to_string(), "SAMPLE-002".to_string()];
    source.target_sample_count = 2;
    assert!(validate_benchmark_data_source_spec(&source).is_ok());

    source.collections[0].selected_accessions.reverse();
    assert!(validate_benchmark_data_source_spec(&source)
        .is_err_and(|error| error.to_string().contains("strictly ordered")));

    source.collections[0].selected_accessions = vec!["SAMPLE-001".to_string()];
    assert!(validate_benchmark_data_source_spec(&source)
        .is_err_and(|error| error.to_string().contains("planned sample count")));

    source.collections[0].planned_sample_count = Some(1);
    source.collections[1].planned_sample_count = Some(1);
    source.collections[1].selected_accessions = vec!["SAMPLE-001".to_string()];
    assert!(validate_benchmark_data_source_spec(&source)
        .is_err_and(|error| error.to_string().contains("more than once")));
}

#[test]
fn checked_generic_source_schema_matches_model() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/benchmark-data-source.schema.json");
    let generated =
        serde_json::to_value(bijux_dna_bench_model::benchmark_data_source_spec_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated);
    Ok(())
}

fn source_spec() -> BenchmarkDataSourceSpec {
    BenchmarkDataSourceSpec {
        schema_version: BENCHMARK_DATA_SOURCE_SCHEMA_V1.to_string(),
        portfolio_id: "variant-truth-panel".to_string(),
        policy: CorpusSourcePolicy {
            class: BenchmarkCorpusClass::ProductionReal,
            access_classification: "public".to_string(),
            redistribution_classification: "accession-referenced".to_string(),
            access_terms_url: "https://example.org/access".to_string(),
            attribution: "Cite the source collection.".to_string(),
            projected_storage_bytes: 1_000_000,
            projected_cpu_hours: 10,
            projection_basis: "Ten samples with bounded verification work.".to_string(),
            truth_roles: vec!["variant_truth".to_string()],
            truth_strategy: "independent_reference_material".to_string(),
            known_limitations: vec!["The source is not population representative.".to_string()],
        },
        evaluation: CorpusEvaluationDesign {
            sample_context: "Ten reference-material samples.".to_string(),
            assay_context: "Whole-genome sequencing.".to_string(),
            strata: vec!["pedigree_role".to_string(), "sequencing_center".to_string()],
            controls: vec!["mendelian_consistency".to_string()],
            held_out_split: "Mask truth genotypes after stratified sample assignment.".to_string(),
            lineage: "Archive accession to admitted source catalog to derived truth panel."
                .to_string(),
            allowed_uses: vec!["accuracy".to_string(), "performance".to_string()],
        },
        description: "Public reference material for variant benchmarking.".to_string(),
        data_types: vec!["bam".to_string(), "vcf".to_string()],
        target_sample_count: 10,
        collections: vec![
            BenchmarkDataSourceCollection {
                id: "reference-material".to_string(),
                authority: "Reference authority".to_string(),
                accession: "REFERENCE-001".to_string(),
                source_url: "https://example.org/record".to_string(),
                access_terms_url: "https://example.org/access".to_string(),
                data_types: vec!["bam".to_string(), "vcf".to_string()],
                planned_sample_count: Some(10),
                selected_accessions: Vec::new(),
                selection_scope: "All ten independently characterized samples.".to_string(),
                truth_basis: "Orthogonal measurements define benchmark genotypes.".to_string(),
                known_limitations: vec![
                    "Difficult regions remain outside the benchmark.".to_string()
                ],
            },
            BenchmarkDataSourceCollection {
                id: "supporting-map".to_string(),
                authority: "Reference authority".to_string(),
                accession: "MAP-001".to_string(),
                source_url: "https://example.org/map".to_string(),
                access_terms_url: "https://example.org/access".to_string(),
                data_types: vec!["vcf".to_string()],
                planned_sample_count: None,
                selected_accessions: Vec::new(),
                selection_scope: "One immutable supporting map.".to_string(),
                truth_basis: "The authority publishes the map independently of samples."
                    .to_string(),
                known_limitations: vec!["The map is population specific.".to_string()],
            },
        ],
    }
}
