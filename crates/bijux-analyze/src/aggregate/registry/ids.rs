//! Owner: bijux-analyze
//! Metric ids and specs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageMetricKind {
    FastqTrim,
    FastqValidate,
    FastqFilter,
    FastqMerge,
    FastqCorrect,
    FastqQcPost,
    FastqUmi,
    FastqScreen,
    FastqStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricId {
    RuntimeS,
    MemoryMb,
    ExitCode,
    ReadsIn,
    ReadsOut,
    ReadsDropped,
    ReadsRemovedByN,
    ReadsRemovedByEntropy,
    ReadsRemovedLowComplexity,
    ReadsRemovedByKmer,
    ReadsRemovedContaminantKmer,
    ReadsRemovedByLength,
    ReadsTotal,
    ReadsValid,
    ReadsInvalid,
    BasesIn,
    BasesOut,
    BasesTotal,
    PairsIn,
    PairsOut,
    ReadsR1,
    ReadsR2,
    ReadsMerged,
    ReadsUnmerged,
    MeanQBefore,
    MeanQAfter,
    MeanQ,
    MergeRate,
    DedupRate,
    KmerFixRate,
    ContaminationRate,
    ContaminationSummary,
    GcPercent,
    LengthHistogram,
    DeltaMetrics,
    AdapterPreset,
    AdapterBankId,
    AdapterBankHash,
    AdapterOverrides,
    QcRawDir,
    QcTrimmedDir,
    MultiqcReport,
    MultiqcData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DerivedMetricId {
    ReadRetention,
    BaseRetention,
    MergeEfficiency,
    ErrorReductionProxy,
}

#[derive(Debug, Clone, Copy)]
pub enum MetricDirection {
    HigherBetter,
    LowerBetter,
    Neutral,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricSpec {
    pub id: MetricId,
    pub name: &'static str,
    pub meaning: &'static str,
    pub direction: MetricDirection,
    pub range: Option<MetricRange>,
    pub stages: &'static [&'static str],
    pub measured: bool,
    pub derived: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct DerivedMetricSpec {
    pub id: DerivedMetricId,
    pub name: &'static str,
    pub meaning: &'static str,
    pub direction: MetricDirection,
    pub range: Option<MetricRange>,
    pub stages: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct StageMetricSpec {
    pub stage: &'static str,
    pub version: i32,
    pub metrics: &'static [MetricId],
    pub invariants: &'static [&'static str],
}
