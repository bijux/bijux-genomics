//! Owner: bijux-dna-analyze
//! Report section builders.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_core::prelude::{InvariantStatusV1, RawFailure};
use bijux_dna_runtime::{FactsRowV1, PipelineVerdictV1, StageReportV1, TelemetryEventV1};

use crate::decision::score::{build_rankings, RankInput};
use crate::failure::{classify_raw_failure, BenchmarkFailure};

mod run_overview;
mod findings;
mod metrics;
mod qc;
pub mod schema;

pub(crate) use run_overview::*;
pub(crate) use findings::*;
pub(crate) use metrics::*;
pub(crate) use qc::*;
