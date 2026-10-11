use bijux_dna_bench_model::{
    benchmark_protocol_citation_schema, benchmark_run_case_index_schema,
    benchmark_run_index_schema, benchmark_run_publication_pointer_schema,
    validate_benchmark_protocol_citation, validate_benchmark_run_case_index,
    validate_benchmark_run_index, validate_benchmark_run_publication_pointer,
    BenchmarkProtocolCitation, BenchmarkProtocolCitationStatus, BenchmarkRunCaseIndex,
    BenchmarkRunIdentity, BenchmarkRunIndex, BenchmarkRunIndexEntry,
    BenchmarkRunIndexFileReference, BenchmarkRunIndexSummary, BenchmarkRunPublicationPointer,
    BenchmarkRunRegistryState, BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1,
    BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1, BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1,
};

fn run_index() -> BenchmarkRunIndex {
    BenchmarkRunIndex::v1(
        BenchmarkRunIndexFileReference {
            uri: "repo://benchmarks/contracts/run-registry.json".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 1,
        },
        BenchmarkRunIndexSummary {
            registered_run_count: 1,
            active_protocol_count: 1,
            reserved_protocol_count: 0,
            retired_protocol_count: 0,
            citable_publication_count: 0,
            unpublished_run_count: 1,
        },
        vec![BenchmarkRunIndexEntry {
            run_id: "run-2001".to_string(),
            citation_key: "bijux-benchmark-run-2001".to_string(),
            protocol_title: "BAM genotyping benchmark protocol".to_string(),
            slug: "bam-genotyping".to_string(),
            stage_id: "bam.genotyping".to_string(),
            owner: "bijux-dna-bench".to_string(),
            protocol_state: BenchmarkRunRegistryState::Active,
            protocol_sha256: "b".repeat(64),
            registry_uri: "repo://benchmarks/contracts/run-registry.json#run-2001".to_string(),
            tool_ids: vec!["bcftools".to_string()],
            corpus_roles: vec!["variant_truth".to_string()],
            acceptance_policy_id: "bam-biological-inference-acceptance-v1".to_string(),
            selected_publication: None,
            limitations: vec![
                "No current admitted publication is selected for this protocol.".to_string()
            ],
        }],
    )
}

#[test]
fn unpublished_run_index_is_complete_and_explicit() -> anyhow::Result<()> {
    let index = run_index();
    validate_benchmark_run_index(&index)?;
    let actual: BenchmarkRunIndex = serde_json::from_slice(&serde_json::to_vec(&index)?)?;
    assert_eq!(actual, index);
    Ok(())
}

#[test]
fn run_index_rejects_summary_drift_and_hidden_publication_absence() {
    let mut summary_drift = run_index();
    summary_drift.summary.citable_publication_count = 1;
    assert!(validate_benchmark_run_index(&summary_drift).is_err());

    let mut hidden_absence = run_index();
    hidden_absence.runs[0].limitations.clear();
    assert!(validate_benchmark_run_index(&hidden_absence).is_err());
}

#[test]
fn run_index_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(run_index())?;
    value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("run index must be an object"))?
        .insert("unexpected".to_string(), serde_json::json!(true));
    assert!(serde_json::from_value::<BenchmarkRunIndex>(value).is_err());
    Ok(())
}

#[test]
fn checked_run_index_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))?;
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository_root.join("benchmarks/contracts/schemas/run-index.schema.json"),
    )?)?;
    assert_eq!(checked, serde_json::to_value(benchmark_run_index_schema())?);
    for (path, schema) in [
        ("protocol-citation.schema.json", benchmark_protocol_citation_schema()),
        ("run-case-index.schema.json", benchmark_run_case_index_schema()),
        ("run-publication-pointer.schema.json", benchmark_run_publication_pointer_schema()),
    ] {
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(
            repository_root.join("benchmarks/contracts/schemas").join(path),
        )?)?;
        assert_eq!(checked, serde_json::to_value(schema)?);
    }
    Ok(())
}

#[test]
fn unpublished_protocol_surfaces_are_explicit_and_consistent() -> anyhow::Result<()> {
    let run = BenchmarkRunIdentity {
        run_id: "run-2001".to_string(),
        domain: "bam".to_string(),
        stage_id: "bam.genotyping".to_string(),
    };
    let limitation = vec!["No admitted publication is selected.".to_string()];
    let citation = BenchmarkProtocolCitation {
        schema_version: BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1.to_string(),
        citation_key: "bijux-benchmark-run-2001".to_string(),
        run: run.clone(),
        protocol_title: "BAM genotyping benchmark protocol".to_string(),
        protocol_version: "1.0.0".to_string(),
        protocol_sha256: "a".repeat(64),
        status: BenchmarkProtocolCitationStatus::ProtocolOnly,
        selected_publication: None,
        limitations: limitation.clone(),
    };
    let cases = BenchmarkRunCaseIndex {
        schema_version: BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1.to_string(),
        run: run.clone(),
        protocol_sha256: "a".repeat(64),
        cases: Vec::new(),
        limitations: limitation.clone(),
    };
    let publication = BenchmarkRunPublicationPointer {
        schema_version: BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1.to_string(),
        run,
        protocol_sha256: "a".repeat(64),
        selected_publication: None,
        limitations: limitation,
    };

    validate_benchmark_protocol_citation(&citation)?;
    validate_benchmark_run_case_index(&cases)?;
    validate_benchmark_run_publication_pointer(&publication)?;

    let mut contradictory = citation;
    contradictory.status = BenchmarkProtocolCitationStatus::Published;
    assert!(validate_benchmark_protocol_citation(&contradictory).is_err());
    Ok(())
}
