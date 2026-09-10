//! Owner: micro-benchmark acceptance over typed rows and observed file presence.
use crate::{
    MicroBenchmarkExecutionStatus, MicroBenchmarkLogRow, MicroBenchmarkNormalizedMetricRow,
    MicroBenchmarkOutputRow, MicroBenchmarkResultRow, MicroBenchmarkRunManifest,
};
use anyhow::{anyhow, bail, Result};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub struct MicroBenchmarkRunEvidence<'a> {
    pub results: &'a [MicroBenchmarkResultRow],
    pub outputs: &'a [MicroBenchmarkOutputRow],
    pub logs: &'a [MicroBenchmarkLogRow],
    pub metrics: &'a [MicroBenchmarkNormalizedMetricRow],
}

/// Validate run declarations against rows and a current observation of required files.
/// File presence alone does not establish scientific equivalence or successful tool execution.
///
/// # Errors
/// Rejects missing required components/evidence, contradictory counts or unsupported metric claims.
pub fn validate_micro_benchmark_run_contract(
    manifest: &MicroBenchmarkRunManifest,
    evidence: MicroBenchmarkRunEvidence<'_>,
    present_files: &BTreeSet<String>,
) -> Result<()> {
    validate_manifest(manifest, evidence)?;
    validate_components(manifest, present_files)?;
    validate_results(evidence.results, present_files)?;
    validate_observation_rows(evidence)
}

fn validate_manifest(
    manifest: &MicroBenchmarkRunManifest,
    evidence: MicroBenchmarkRunEvidence<'_>,
) -> Result<()> {
    if manifest.schema_version != crate::MICRO_BENCHMARK_RUN_SCHEMA_VERSION {
        bail!("unsupported micro-benchmark run schema");
    }
    if manifest.run_root != "artifacts/benchmark-smoke/micro" {
        bail!(
            "micro benchmark run root must stay `artifacts/benchmark-smoke/micro`, found `{}`",
            manifest.run_root
        );
    }
    if manifest.component_reports.len() < 7 {
        bail!(
            "micro benchmark run must keep at least seven component reports, found {}",
            manifest.component_reports.len()
        );
    }
    for (empty, name) in [
        (evidence.results.is_empty(), "result"),
        (evidence.outputs.is_empty(), "output"),
        (evidence.logs.is_empty(), "log"),
        (evidence.metrics.is_empty(), "normalized metric"),
    ] {
        if empty {
            bail!("micro benchmark run must write at least one {name} row");
        }
    }
    if manifest.result_row_count != evidence.results.len()
        || manifest.output_row_count != evidence.outputs.len()
        || manifest.log_row_count != evidence.logs.len()
        || manifest.normalized_metric_row_count != evidence.metrics.len()
    {
        bail!("micro benchmark manifest counts drifted from written row sets");
    }
    Ok(())
}

fn validate_components(
    manifest: &MicroBenchmarkRunManifest,
    present_files: &BTreeSet<String>,
) -> Result<()> {
    for component in &manifest.component_reports {
        if !present_files.contains(&component.report_path) {
            bail!(
                "micro benchmark component report `{}` is missing at `{}`",
                component.component_id,
                component.report_path
            );
        }
        let path = std::path::Path::new(&component.report_path);
        if component.report_path.contains('\\')
            || !component.report_path.starts_with("artifacts/benchmark-smoke/micro/")
            || !path.components().all(|part| matches!(part, std::path::Component::Normal(_)))
        {
            bail!("micro benchmark component report `{}` must stay under `artifacts/benchmark-smoke/micro`, found `{}`", component.component_id, component.report_path);
        }
    }
    let component_ids =
        manifest.component_reports.iter().map(|c| c.component_id.as_str()).collect::<BTreeSet<_>>();
    let expected_component_ids = BTreeSet::from([
        "amplicon_micro_pipeline",
        "adna_micro_pipeline",
        "bam_micro_smoke_subset",
        "core_germline_micro_pipeline",
        "edna_micro_pipeline",
        "fastq_micro_smoke_subset",
        "real_smoke_core_subset",
        "vcf_micro_smoke_subset",
    ]);
    if !expected_component_ids.is_subset(&component_ids) {
        bail!("micro benchmark component reports drifted: observed={component_ids:?} expected_at_least={expected_component_ids:?}");
    }
    Ok(())
}

fn validate_results(
    results: &[MicroBenchmarkResultRow],
    present_files: &BTreeSet<String>,
) -> Result<()> {
    for row in results {
        match row.status {
            MicroBenchmarkExecutionStatus::Succeeded => {
                let path = row.evidence_path.as_ref().ok_or_else(|| {
                    anyhow!(
                        "successful micro benchmark row `{}` must keep an evidence path",
                        row.execution_id
                    )
                })?;
                if !present_files.contains(path) {
                    bail!(
                        "successful micro benchmark row `{}` is missing evidence `{}`",
                        row.execution_id,
                        path
                    );
                }
            }
            MicroBenchmarkExecutionStatus::ContainerNeeded
            | MicroBenchmarkExecutionStatus::Unavailable => {
                if row.normalized_metric_count != 0 {
                    bail!(
                        "non-succeeded micro benchmark row `{}` must not claim normalized metrics",
                        row.execution_id
                    );
                }
            }
        }
    }
    Ok(())
}

fn validate_observation_rows(evidence: MicroBenchmarkRunEvidence<'_>) -> Result<()> {
    for row in evidence.outputs {
        if !row.exists {
            bail!(
                "micro benchmark output row `{}` references missing path `{}`",
                row.execution_id,
                row.path
            );
        }
    }
    for row in evidence.logs {
        if !row.exists {
            bail!(
                "micro benchmark log row `{}` references missing path `{}`",
                row.execution_id,
                row.path
            );
        }
    }
    if !evidence
        .metrics
        .iter()
        .any(|row| row.execution_id == "vcf.call" || row.execution_id == "vcf.stats")
    {
        bail!("micro benchmark normalized metrics must include at least one VCF execution");
    }
    if !evidence
        .results
        .iter()
        .any(|row| row.component_id == "fastq_micro_smoke_subset" && row.domain == "fastq")
    {
        bail!("micro benchmark results must include at least one FASTQ family representative");
    }
    if !evidence
        .results
        .iter()
        .any(|row| row.component_id == "vcf_micro_smoke_subset" && row.domain == "vcf")
    {
        bail!("micro benchmark results must include at least one VCF family representative");
    }
    Ok(())
}
