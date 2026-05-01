use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChunkedPreprocessChunkV1 {
    pub chunk_id: String,
    pub ordinal: u32,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChunkedPreprocessContractV1 {
    pub schema_version: String,
    pub pair_sync_required: bool,
    pub deterministic_concatenation_order: Vec<String>,
    pub artifact_lineage_strategy: String,
    pub report_aggregation_strategy: String,
    pub chunks: Vec<ChunkedPreprocessChunkV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChunkedPreprocessAggregateV1 {
    pub chunk_ids: Vec<String>,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
}

/// # Errors
/// Returns an error if chunk ordinals are duplicated, concatenation order drifts, or pair counts
/// are inconsistent under pair-synchronization requirements.
pub fn aggregate_chunked_preprocess(
    contract: &ChunkedPreprocessContractV1,
) -> Result<ChunkedPreprocessAggregateV1> {
    if contract.chunks.is_empty() {
        return Err(anyhow!("chunked preprocess contract contains no chunks"));
    }
    let mut chunks = contract.chunks.clone();
    chunks.sort_by_key(|chunk| chunk.ordinal);

    let actual_order = chunks.iter().map(|chunk| chunk.chunk_id.clone()).collect::<Vec<_>>();
    if actual_order != contract.deterministic_concatenation_order {
        return Err(anyhow!(
            "chunk concatenation order does not match declared deterministic order"
        ));
    }

    let mut reads_in = 0_u64;
    let mut reads_out = 0_u64;
    let mut bases_in = 0_u64;
    let mut bases_out = 0_u64;
    let mut pairs_in = 0_u64;
    let mut pairs_out = 0_u64;
    let mut has_pairs = false;

    for (index, chunk) in chunks.iter().enumerate() {
        if chunk.ordinal != index as u32 {
            return Err(anyhow!("chunk ordinals must be contiguous from zero"));
        }
        if contract.pair_sync_required {
            let chunk_pairs_in = chunk
                .pairs_in
                .ok_or_else(|| anyhow!("paired chunk {} missing pairs_in", chunk.chunk_id))?;
            let chunk_pairs_out = chunk
                .pairs_out
                .ok_or_else(|| anyhow!("paired chunk {} missing pairs_out", chunk.chunk_id))?;
            if chunk.output_r2.is_none() {
                return Err(anyhow!("paired chunk {} missing output_r2", chunk.chunk_id));
            }
            pairs_in += chunk_pairs_in;
            pairs_out += chunk_pairs_out;
            has_pairs = true;
        }
        reads_in += chunk.reads_in;
        reads_out += chunk.reads_out;
        bases_in += chunk.bases_in;
        bases_out += chunk.bases_out;
    }

    Ok(ChunkedPreprocessAggregateV1 {
        chunk_ids: actual_order,
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in: has_pairs.then_some(pairs_in),
        pairs_out: has_pairs.then_some(pairs_out),
    })
}

#[must_use]
pub fn chunked_and_unchunked_are_equivalent(
    chunked: &ChunkedPreprocessAggregateV1,
    unchunked: &ChunkedPreprocessAggregateV1,
) -> bool {
    chunked.reads_in == unchunked.reads_in
        && chunked.reads_out == unchunked.reads_out
        && chunked.bases_in == unchunked.bases_in
        && chunked.bases_out == unchunked.bases_out
        && chunked.pairs_in == unchunked.pairs_in
        && chunked.pairs_out == unchunked.pairs_out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_chunked_preprocess_rejects_order_drift() {
        let contract = ChunkedPreprocessContractV1 {
            schema_version: "bijux.fastq.chunked_preprocess.v1".to_string(),
            pair_sync_required: true,
            deterministic_concatenation_order: vec!["chunk-b".to_string(), "chunk-a".to_string()],
            artifact_lineage_strategy: "concatenate_by_declared_chunk_order".to_string(),
            report_aggregation_strategy: "sum_chunk_metrics".to_string(),
            chunks: vec![
                ChunkedPreprocessChunkV1 {
                    chunk_id: "chunk-a".to_string(),
                    ordinal: 0,
                    output_r1: "chunk-a.R1.fastq.gz".to_string(),
                    output_r2: Some("chunk-a.R2.fastq.gz".to_string()),
                    reads_in: 10,
                    reads_out: 8,
                    bases_in: 100,
                    bases_out: 80,
                    pairs_in: Some(5),
                    pairs_out: Some(4),
                },
                ChunkedPreprocessChunkV1 {
                    chunk_id: "chunk-b".to_string(),
                    ordinal: 1,
                    output_r1: "chunk-b.R1.fastq.gz".to_string(),
                    output_r2: Some("chunk-b.R2.fastq.gz".to_string()),
                    reads_in: 10,
                    reads_out: 8,
                    bases_in: 100,
                    bases_out: 80,
                    pairs_in: Some(5),
                    pairs_out: Some(4),
                },
            ],
        };

        let err =
            aggregate_chunked_preprocess(&contract).expect_err("order drift must be rejected");
        assert!(err.to_string().contains("deterministic order"));
    }
}
