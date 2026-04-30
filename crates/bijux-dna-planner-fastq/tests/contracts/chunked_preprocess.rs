use anyhow::Result;
use std::path::Path;

#[test]
fn chunked_preprocess_contract_preserves_pair_sync_and_deterministic_order() -> Result<()> {
    let contract = bijux_dna_planner_fastq::plan_chunked_preprocess_contract(
        Path::new("out"),
        true,
        &[
            bijux_dna_planner_fastq::ChunkedFastqInput {
                chunk_id: "chunk-000".to_string(),
                output_r1: "chunk-000.R1.fastq.gz".to_string(),
                output_r2: Some("chunk-000.R2.fastq.gz".to_string()),
                reads_in: 20,
                reads_out: 18,
                bases_in: 200,
                bases_out: 180,
                pairs_in: Some(10),
                pairs_out: Some(9),
            },
            bijux_dna_planner_fastq::ChunkedFastqInput {
                chunk_id: "chunk-001".to_string(),
                output_r1: "chunk-001.R1.fastq.gz".to_string(),
                output_r2: Some("chunk-001.R2.fastq.gz".to_string()),
                reads_in: 20,
                reads_out: 16,
                bases_in: 200,
                bases_out: 150,
                pairs_in: Some(10),
                pairs_out: Some(8),
            },
        ],
    )?;

    let aggregate = bijux_dna_domain_fastq::aggregate_chunked_preprocess(&contract)?;
    assert_eq!(
        contract.deterministic_concatenation_order,
        vec!["chunk-000".to_string(), "chunk-001".to_string()]
    );
    assert_eq!(aggregate.pairs_in, Some(20));
    assert_eq!(aggregate.pairs_out, Some(17));
    Ok(())
}

#[test]
fn chunked_and_unchunked_preprocess_reports_are_equivalent_when_totals_match() -> Result<()> {
    let contract = bijux_dna_planner_fastq::plan_chunked_preprocess_contract(
        Path::new("out"),
        true,
        &[
            bijux_dna_planner_fastq::ChunkedFastqInput {
                chunk_id: "chunk-000".to_string(),
                output_r1: "chunk-000.R1.fastq.gz".to_string(),
                output_r2: Some("chunk-000.R2.fastq.gz".to_string()),
                reads_in: 20,
                reads_out: 18,
                bases_in: 200,
                bases_out: 180,
                pairs_in: Some(10),
                pairs_out: Some(9),
            },
            bijux_dna_planner_fastq::ChunkedFastqInput {
                chunk_id: "chunk-001".to_string(),
                output_r1: "chunk-001.R1.fastq.gz".to_string(),
                output_r2: Some("chunk-001.R2.fastq.gz".to_string()),
                reads_in: 20,
                reads_out: 16,
                bases_in: 200,
                bases_out: 150,
                pairs_in: Some(10),
                pairs_out: Some(8),
            },
        ],
    )?;
    let chunked = bijux_dna_domain_fastq::aggregate_chunked_preprocess(&contract)?;
    let unchunked = bijux_dna_domain_fastq::ChunkedPreprocessAggregateV1 {
        chunk_ids: vec!["unchunked".to_string()],
        reads_in: 40,
        reads_out: 34,
        bases_in: 400,
        bases_out: 330,
        pairs_in: Some(20),
        pairs_out: Some(17),
    };

    assert!(bijux_dna_domain_fastq::chunked_and_unchunked_are_equivalent(&chunked, &unchunked));
    Ok(())
}
