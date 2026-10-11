use anyhow::{bail, Result};
use bijux_dna_bench_vcf_evidence::model::outputs::{
    VcfAdapterOutputCoverageRow, VcfAdapterOutputCoverageStatus,
};
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_adapter_output_coverage_contract(
    rows: &[VcfAdapterOutputCoverageRow],
    benchmark_ready_pairs: &BTreeSet<(String, String)>,
) -> Result<()> {
    let mut seen = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen.insert((row.stage_id.as_str(), row.tool_id.as_str())) {
            bail!(
                "VCF adapter output coverage contains duplicate row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            );
        }
    }

    let actual_benchmark_ready_pairs = rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();
    if &actual_benchmark_ready_pairs != benchmark_ready_pairs {
        bail!(
            "VCF adapter output coverage drifted from the benchmark-ready matrix rows: expected {benchmark_ready_pairs:?}, found {actual_benchmark_ready_pairs:?}"
        );
    }

    for row in rows.iter().filter(|row| row.benchmark_status == "benchmark_ready") {
        if row.status != VcfAdapterOutputCoverageStatus::Complete {
            bail!(
                "benchmark-ready VCF row `{}` / `{}` is missing governed output declarations: {}",
                row.stage_id,
                row.tool_id,
                row.missing_declarations.join(", ")
            );
        }
    }

    Ok(())
}
