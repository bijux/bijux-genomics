use bijux_dna_bench_model::{
    validate_benchmark_run_index, BenchmarkRunIndex, BenchmarkRunIndexEntry,
    BenchmarkRunIndexFileReference, BenchmarkRunIndexSummary, BenchmarkRunRegistryState,
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
