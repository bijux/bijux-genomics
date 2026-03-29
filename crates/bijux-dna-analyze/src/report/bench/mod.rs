//! Owner: bijux-dna-analyze
//! Benchmark report helpers and exporters.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::RawFailure;
use bijux_dna_infra::atomic_write_bytes;

use crate::aggregate::*;
use crate::decision::score::{build_rankings, RankInput, RankingEntry};
use crate::failure::{classify_raw_failure, BenchmarkFailure};

mod export;
mod recommendations;
mod summary;

pub use export::*;
pub use recommendations::*;
pub use summary::*;
