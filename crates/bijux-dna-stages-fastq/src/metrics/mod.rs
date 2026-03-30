use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::id_catalog;
use flate2::read::GzDecoder;

use bijux_dna_core::contract::canonical::parameters_json_canonicalization;
use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::contract::MetricProvenanceV1;
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_dna_domain_fastq::parse_effective_params;
use bijux_dna_stage_contract::StagePlanV1;

mod envelope_support;
mod fastqc;
mod filters;
mod stage_metrics;
mod stage_metrics_analysis;
mod stage_metrics_reporting;
mod stage_metrics_transform;

pub(crate) use envelope_support::{
    build_metrics_envelope, f64_from_u64, pair_counts_from_paths,
    retention_conditions_from_effective, stats_for_paths, zero_seqkit_metrics,
};
