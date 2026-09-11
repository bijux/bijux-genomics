use super::ToolServingMapRow;
use anyhow::{anyhow, Result};
use bijux_dna_core::id_catalog::{
    FASTQ_CLUSTER_OTUS, FASTQ_INFER_ASVS, FASTQ_NORMALIZE_ABUNDANCE, FASTQ_NORMALIZE_PRIMERS,
    FASTQ_REMOVE_CHIMERAS,
};
pub(super) fn ensure_fastq_amplicon_fixture_coverage(rows: &[ToolServingMapRow]) -> Result<()> {
    let expected_rows = [
        (
            "cutadapt",
            FASTQ_NORMALIZE_PRIMERS,
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "vsearch",
            FASTQ_REMOVE_CHIMERAS,
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        ("dada2", FASTQ_INFER_ASVS, "governed_execution", "runnable", "parse_normalized"),
        (
            "vsearch",
            FASTQ_CLUSTER_OTUS,
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "seqkit",
            FASTQ_NORMALIZE_ABUNDANCE,
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
    ];

    for (tool_id, stage_id, support_status, adapter_status, parser_status) in expected_rows {
        let row =
            rows.iter().find(|row| row.tool_id == tool_id && row.stage_id == stage_id).ok_or_else(
                || anyhow!("FASTQ amplicon serving map is missing `{stage_id}` / `{tool_id}`"),
            )?;
        if row.support_status != support_status
            || row.adapter_status != adapter_status
            || row.parser_status != parser_status
            || row.corpus_status != "fixture:corpus-03-amplicon-mini"
        {
            return Err(anyhow!(
                "FASTQ amplicon serving row `{stage_id}` / `{tool_id}` must keep its governed corpus-03 readiness contract"
            ));
        }
    }

    Ok(())
}
