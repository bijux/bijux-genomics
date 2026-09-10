//! Owner: observe the current required micro-run component and result files.
use bijux_dna_bench_model::{
    MicroBenchmarkExecutionStatus, MicroBenchmarkRunEvidence, MicroBenchmarkRunManifest,
};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn present_files(
    root: &Path,
    manifest: &MicroBenchmarkRunManifest,
    evidence: MicroBenchmarkRunEvidence<'_>,
) -> BTreeSet<String> {
    manifest
        .component_reports
        .iter()
        .map(|row| row.report_path.as_str())
        .chain(
            evidence
                .results
                .iter()
                .filter(|row| row.status == MicroBenchmarkExecutionStatus::Succeeded)
                .filter_map(|row| row.evidence_path.as_deref()),
        )
        .filter(|path| root.join(path).is_file())
        .map(str::to_string)
        .collect()
}
