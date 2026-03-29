//! Owner: bijux-dna-analyze
//! Benchmark report helpers and exporters.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::RawFailure;
use bijux_dna_infra::atomic_write_bytes;

use crate::aggregate::{
    derived_metric_spec, derived_metrics_for_stage, metric_kind_for_stage, metric_spec,
    stage_metric_spec, BenchmarkRecord, DerivedMetricId, FastqClusterOtusMetrics,
    FastqCorrectMetrics, FastqDepleteHostMetrics, FastqDepleteReferenceContaminantsMetrics,
    FastqDepleteRrnaMetrics, FastqDetectAdaptersMetrics, FastqFilterMetrics,
    FastqIndexReferenceMetrics, FastqInferAsvsMetrics, FastqLowComplexityMetrics,
    FastqMergeMetrics, FastqNormalizeAbundanceMetrics, FastqNormalizePrimersMetrics,
    FastqOverrepresentedMetrics, FastqQcPostMetrics, FastqReadLengthMetrics, FastqScreenMetrics,
    FastqStatsMetrics, FastqTrimMetrics, FastqTrimPolygMetrics, FastqTrimTerminalDamageMetrics,
    FastqUmiMetrics, FastqValidateMetrics,
};
use crate::decision::score::{build_rankings, RankInput, RankingEntry};
use crate::failure::{classify_raw_failure, BenchmarkFailure};

mod export;
mod recommendations;
mod summary;

pub use export::*;
pub use recommendations::*;
pub use summary::*;
