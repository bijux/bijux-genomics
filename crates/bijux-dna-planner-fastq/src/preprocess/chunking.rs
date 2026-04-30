use std::path::Path;

use anyhow::Result;
use bijux_dna_domain_fastq::{ChunkedPreprocessChunkV1, ChunkedPreprocessContractV1};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkedFastqInput {
    pub chunk_id: String,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
}

/// # Errors
/// Returns an error if the resulting chunk contract fails governed aggregation validation.
pub fn plan_chunked_preprocess_contract(
    _out_dir: &Path,
    pair_sync_required: bool,
    chunks: &[ChunkedFastqInput],
) -> Result<ChunkedPreprocessContractV1> {
    let contract = ChunkedPreprocessContractV1 {
        schema_version: "bijux.fastq.chunked_preprocess.v1".to_string(),
        pair_sync_required,
        deterministic_concatenation_order: chunks
            .iter()
            .map(|chunk| chunk.chunk_id.clone())
            .collect(),
        artifact_lineage_strategy: "concatenate_by_declared_chunk_order".to_string(),
        report_aggregation_strategy: "sum_chunk_metrics".to_string(),
        chunks: chunks
            .iter()
            .enumerate()
            .map(|(ordinal, chunk)| ChunkedPreprocessChunkV1 {
                chunk_id: chunk.chunk_id.clone(),
                ordinal: ordinal as u32,
                output_r1: chunk.output_r1.clone(),
                output_r2: chunk.output_r2.clone(),
                reads_in: chunk.reads_in,
                reads_out: chunk.reads_out,
                bases_in: chunk.bases_in,
                bases_out: chunk.bases_out,
                pairs_in: chunk.pairs_in,
                pairs_out: chunk.pairs_out,
            })
            .collect(),
    };
    bijux_dna_domain_fastq::aggregate_chunked_preprocess(&contract)?;
    Ok(contract)
}
