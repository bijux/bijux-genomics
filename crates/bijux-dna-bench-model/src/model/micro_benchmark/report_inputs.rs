//! Owner: typed, non-persisted inputs to micro-benchmark report assembly.
use crate::{
    MicroBenchmarkInsufficientDataRow, MicroBenchmarkLogRow, MicroBenchmarkMemorySourceRow,
    MicroBenchmarkOutputRow, MicroBenchmarkResultRow, MicroBenchmarkRunManifest,
    MicroBenchmarkRuntimeRow, MicroBenchmarkScienceThresholdRow,
};
use std::collections::BTreeSet;
/// Expected execution projected by an upstream component adapter.
#[derive(Debug, Clone)]
pub struct MicroBenchmarkExpectedRow {
    pub component_id: String,
    pub execution_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub source_report_path: String,
}

/// Borrowed run observations and the expected component coverage.
#[derive(Clone, Copy)]
pub struct MicroBenchmarkReportSources<'a> {
    pub manifest: &'a MicroBenchmarkRunManifest,
    pub results: &'a [MicroBenchmarkResultRow],
    pub outputs: &'a [MicroBenchmarkOutputRow],
    pub logs: &'a [MicroBenchmarkLogRow],
    pub expected: &'a [MicroBenchmarkExpectedRow],
}
/// Repository observations captured for one report request; no scientific verdict is implied.
pub struct MicroBenchmarkReportObservations {
    pub existing_evidence_paths: BTreeSet<String>,
    pub insufficient: Vec<MicroBenchmarkInsufficientDataRow>,
    pub runtime: Vec<MicroBenchmarkRuntimeRow>,
    pub memory: Vec<MicroBenchmarkMemorySourceRow>,
    pub thresholds: Vec<MicroBenchmarkScienceThresholdRow>,
}
/// Admitted display paths written into the report.
pub struct MicroBenchmarkReportDestinations {
    pub markdown: String,
    pub json: String,
}
