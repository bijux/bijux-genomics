use std::path::Path;

use anyhow::{anyhow, Result};

use crate::chunking::{
    aggregate_chunked_preprocess, chunked_and_unchunked_are_equivalent, ChunkedPreprocessAggregateV1,
    ChunkedPreprocessChunkV1, ChunkedPreprocessContractV1,
};

use super::fastq_io::{read_fastq_records, write_fastq_records};

/// Materialize deterministic FASTQ chunks for chunked preprocessing workflows.
///
/// # Errors
/// Returns an error when inputs are incoherent or chunk artifacts cannot be written.
pub fn build_chunked_preprocess_contract(
    r1: &Path,
    r2: Option<&Path>,
    chunk_size_reads_per_stream: usize,
    chunk_dir: &Path,
) -> Result<ChunkedPreprocessContractV1> {
    if chunk_size_reads_per_stream == 0 {
        return Err(anyhow!("chunk size must be greater than zero"));
    }

    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        read_fastq_records(path)?
    } else {
        Vec::new()
    };
    let paired = r2.is_some();
    if paired && left.len() != right.len() {
        return Err(anyhow!(
            "fastq.chunk_preprocess refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }

    std::fs::create_dir_all(chunk_dir)?;

    let mut chunks = Vec::<ChunkedPreprocessChunkV1>::new();
    let mut deterministic_order = Vec::<String>::new();

    let total = left.len();
    let mut offset = 0_usize;
    let mut ordinal = 0_u32;
    while offset < total {
        let end = usize::min(offset + chunk_size_reads_per_stream, total);
        let id = format!("chunk-{:04}", ordinal + 1);
        let r1_path = chunk_dir.join(format!("{}.R1.fastq.gz", id));
        let r2_path = paired.then(|| chunk_dir.join(format!("{}.R2.fastq.gz", id)));

        let chunk_left = &left[offset..end];
        let chunk_right = if paired { &right[offset..end] } else { &[] };

        write_fastq_records(&r1_path, chunk_left)?;
        if let Some(path) = &r2_path {
            write_fastq_records(path, chunk_right)?;
        }

        let reads_in = if paired {
            (chunk_left.len() + chunk_right.len()) as u64
        } else {
            chunk_left.len() as u64
        };
        let bases_in = chunk_left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
            + chunk_right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

        chunks.push(ChunkedPreprocessChunkV1 {
            chunk_id: id.clone(),
            ordinal,
            output_r1: r1_path.display().to_string(),
            output_r2: r2_path.as_ref().map(|path| path.display().to_string()),
            reads_in,
            reads_out: reads_in,
            bases_in,
            bases_out: bases_in,
            pairs_in: paired.then_some(chunk_left.len() as u64),
            pairs_out: paired.then_some(chunk_left.len() as u64),
        });
        deterministic_order.push(id);

        offset = end;
        ordinal += 1;
    }

    Ok(ChunkedPreprocessContractV1 {
        schema_version: "bijux.fastq.chunked_preprocess.v1".to_string(),
        pair_sync_required: paired,
        deterministic_concatenation_order: deterministic_order,
        artifact_lineage_strategy: "concatenate_by_declared_chunk_order".to_string(),
        report_aggregation_strategy: "sum_chunk_metrics".to_string(),
        chunks,
    })
}

/// Verify chunked preprocessing aggregate against unchunked aggregate metrics.
///
/// # Errors
/// Returns an error if chunk aggregation fails or equivalence check fails.
pub fn verify_chunked_preprocess_equivalence(
    contract: &ChunkedPreprocessContractV1,
    unchunked: &ChunkedPreprocessAggregateV1,
) -> Result<ChunkedPreprocessAggregateV1> {
    let aggregate = aggregate_chunked_preprocess(contract)?;
    if !chunked_and_unchunked_are_equivalent(&aggregate, unchunked) {
        return Err(anyhow!(
            "chunked preprocess aggregate does not match unchunked aggregate"
        ));
    }
    Ok(aggregate)
}

#[cfg(test)]
mod tests {
    use super::{build_chunked_preprocess_contract, verify_chunked_preprocess_equivalence};
    use crate::chunking::ChunkedPreprocessAggregateV1;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn chunked_preprocess_contract_supports_equivalence_checks() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-chunked-preprocess")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");

        write_fastq(
            &r1,
            &[
                ("a/1", "AAAA", "IIII"),
                ("b/1", "CCCC", "IIII"),
                ("c/1", "GGGG", "IIII"),
                ("d/1", "TTTT", "IIII"),
                ("e/1", "ACGT", "IIII"),
            ],
        )?;
        write_fastq(
            &r2,
            &[
                ("a/2", "TTTT", "IIII"),
                ("b/2", "GGGG", "IIII"),
                ("c/2", "CCCC", "IIII"),
                ("d/2", "AAAA", "IIII"),
                ("e/2", "TGCA", "IIII"),
            ],
        )?;

        let contract = build_chunked_preprocess_contract(
            &r1,
            Some(&r2),
            2,
            &temp.path().join("chunks"),
        )?;
        assert_eq!(contract.chunks.len(), 3);

        let unchunked = ChunkedPreprocessAggregateV1 {
            chunk_ids: vec![],
            reads_in: 10,
            reads_out: 10,
            bases_in: 40,
            bases_out: 40,
            pairs_in: Some(5),
            pairs_out: Some(5),
        };

        let aggregate = verify_chunked_preprocess_equivalence(&contract, &unchunked)?;
        assert_eq!(aggregate.pairs_in, Some(5));
        Ok(())
    }
}
