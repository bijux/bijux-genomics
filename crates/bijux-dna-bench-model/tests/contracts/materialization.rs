use bijux_dna_bench_model::{
    validate_corpus_acquisition_plan, validate_governed_data_catalog, CorpusAcquisitionFile,
    CorpusAcquisitionLayout, CorpusAcquisitionPlan, CorpusAcquisitionSample,
    CorpusMaterializationDisposition, CorpusMaterializationPublication, GovernedDataCatalog,
    GovernedDataCatalogFile, GovernedDataCatalogSample,
    BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1, CORPUS_ACQUISITION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V1,
};

#[test]
fn acquisition_and_admission_contracts_accept_complete_fastq_identity() {
    assert!(validate_corpus_acquisition_plan(&acquisition_plan()).is_ok());
    assert!(validate_governed_data_catalog(&catalog()).is_ok());
}

#[test]
fn acquisition_contract_rejects_aggregate_and_layout_drift() {
    let mut plan = acquisition_plan();
    plan.total_size_bytes += 1;
    assert!(validate_corpus_acquisition_plan(&plan)
        .is_err_and(|error| error.to_string().contains("aggregates")));

    let mut plan = acquisition_plan();
    plan.samples[0].files[0].mate = Some(1);
    assert!(validate_corpus_acquisition_plan(&plan)
        .is_err_and(|error| error.to_string().contains("layout")));
}

#[test]
fn admission_contract_rejects_unknown_samples_and_empty_read_evidence() {
    let mut value = catalog();
    value.files[0].accession = "ERR999999".to_string();
    assert!(validate_governed_data_catalog(&value)
        .is_err_and(|error| error.to_string().contains("unknown sample")));

    let mut value = catalog();
    value.files[0].read_count = 0;
    assert!(validate_governed_data_catalog(&value)
        .is_err_and(|error| error.to_string().contains("must be positive")));

    let mut value = catalog();
    value.samples[0].layout = CorpusAcquisitionLayout::Pe;
    value.files[0].layout = CorpusAcquisitionLayout::Pe;
    value.files[0].mate = Some(1);
    assert!(validate_governed_data_catalog(&value)
        .is_err_and(|error| error.to_string().contains("sample layout")));
}

#[test]
fn persisted_contracts_reject_unknown_fields() -> anyhow::Result<()> {
    let mut plan = serde_json::to_value(acquisition_plan())?;
    plan.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("serialized acquisition plan is not an object"))?
        .insert("unowned".to_string(), true.into());
    assert!(serde_json::from_value::<CorpusAcquisitionPlan>(plan).is_err());

    let mut catalog = serde_json::to_value(catalog())?;
    catalog
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("serialized catalog is not an object"))?
        .insert("unowned".to_string(), true.into());
    assert!(serde_json::from_value::<GovernedDataCatalog>(catalog).is_err());
    Ok(())
}

#[test]
fn materialization_publication_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let publication = CorpusMaterializationPublication {
        schema_version: BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1.to_string(),
        catalog_path: "/study/data/catalog/human-short-read-diversity.json".into(),
        catalog_sha256: "e".repeat(64),
        downloaded_files: 1,
        verified_existing_files: 0,
        catalog: catalog(),
    };
    let value = serde_json::to_value(&publication)?;
    assert_eq!(
        serde_json::from_value::<CorpusMaterializationPublication>(value.clone())?,
        publication
    );
    let mut unsupported = value;
    unsupported["schema_version"] =
        serde_json::json!("bijux.benchmark.corpus_materialization_publication.v2");
    assert!(serde_json::from_value::<CorpusMaterializationPublication>(unsupported).is_err());

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/corpus-materialization-publication.schema.json");
    let generated =
        serde_json::to_value(bijux_dna_bench_model::corpus_materialization_publication_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated);
    Ok(())
}

fn acquisition_plan() -> CorpusAcquisitionPlan {
    CorpusAcquisitionPlan {
        schema_version: CORPUS_ACQUISITION_SCHEMA_V1.to_string(),
        corpus_id: "corpus-01".to_string(),
        portfolio_id: "human-short-read-diversity".to_string(),
        species: "Homo sapiens".to_string(),
        retrieved_at_utc: "2026-08-06T00:00:00+00:00".to_string(),
        source_authority: "European Nucleotide Archive".to_string(),
        source_spec: "benchmarks/corpora/human-short-read-diversity.toml".to_string(),
        source_spec_sha256: "a".repeat(64),
        sample_count: 1,
        file_count: 1,
        total_size_bytes: 42,
        samples: vec![acquisition_sample()],
    }
}

fn acquisition_sample() -> CorpusAcquisitionSample {
    CorpusAcquisitionSample {
        accession: "ERR000001".to_string(),
        study_accession: "PRJEB1".to_string(),
        sample_accession: "SAMEA1".to_string(),
        experiment_accession: "ERX1".to_string(),
        era: "modern".to_string(),
        layout: CorpusAcquisitionLayout::Se,
        size_band: "under-100mb".to_string(),
        selection_reason: "real single-end coverage".to_string(),
        scientific_name: "Homo sapiens".to_string(),
        library_source: "GENOMIC".to_string(),
        library_strategy: "WGS".to_string(),
        library_selection: "RANDOM".to_string(),
        instrument_platform: "ILLUMINA".to_string(),
        instrument_model: "Illumina HiSeq".to_string(),
        archive_base_count: 42,
        archive_read_count: 7,
        first_public: "2020-01-01".to_string(),
        last_updated: "2020-01-02".to_string(),
        source_report_url: "https://www.ebi.ac.uk/ena/portal/api/filereport".to_string(),
        files: vec![CorpusAcquisitionFile {
            mate: None,
            name: "ERR000001.fastq.gz".to_string(),
            source_url: "https://ftp.sra.ebi.ac.uk/ERR000001.fastq.gz".to_string(),
            source_md5: "b".repeat(32),
            size_bytes: 42,
        }],
    }
}

fn catalog() -> GovernedDataCatalog {
    let sample = acquisition_sample();
    GovernedDataCatalog {
        schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V1.to_string(),
        id: "human-short-read-diversity".to_string(),
        purpose: "real human short-read diversity corpus".to_string(),
        source: "European Nucleotide Archive".to_string(),
        source_record: "study://data/evidence/acquisition-plans/plan.json".to_string(),
        source_record_sha256: "c".repeat(64),
        retrieved_at_utc: "2026-08-06T00:00:00+00:00".to_string(),
        admitted_at_utc: "2026-08-06T01:00:00+00:00".to_string(),
        access_classification: "public".to_string(),
        redistribution_classification: "accession-referenced".to_string(),
        access_terms: "ENA public data access terms apply".to_string(),
        scientific_name: "Homo sapiens".to_string(),
        samples: vec![GovernedDataCatalogSample {
            accession: sample.accession.clone(),
            study_accession: sample.study_accession.clone(),
            sample_accession: sample.sample_accession.clone(),
            experiment_accession: sample.experiment_accession.clone(),
            era: sample.era.clone(),
            layout: sample.layout,
            size_band: sample.size_band.clone(),
            selection_reason: sample.selection_reason.clone(),
            scientific_name: sample.scientific_name.clone(),
            library_source: sample.library_source.clone(),
            library_strategy: sample.library_strategy.clone(),
            library_selection: sample.library_selection.clone(),
            instrument_platform: sample.instrument_platform.clone(),
            instrument_model: sample.instrument_model.clone(),
            archive_base_count: sample.archive_base_count,
            archive_read_count: sample.archive_read_count,
            first_public: sample.first_public.clone(),
            last_updated: sample.last_updated.clone(),
        }],
        files: vec![GovernedDataCatalogFile {
            path: "/study/data/corpora/human-short-read-diversity/raw/ERR000001/ERR000001.fastq.gz"
                .to_string(),
            sha256: "d".repeat(64),
            size_bytes: 42,
            kind: "raw-fastq".to_string(),
            accession: sample.accession,
            study_accession: sample.study_accession,
            sample_accession: sample.sample_accession,
            experiment_accession: sample.experiment_accession,
            era: sample.era,
            layout: sample.layout,
            mate: None,
            read_count: 7,
            base_count: 42,
            source_url: sample.files[0].source_url.clone(),
            source_md5: sample.files[0].source_md5.clone(),
            materialization_status: CorpusMaterializationDisposition::Downloaded,
        }],
        known_limitations: vec!["not population representative".to_string()],
    }
}
