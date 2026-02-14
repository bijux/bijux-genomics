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
use bijux_dna_domain_fastq::metrics::*;
use bijux_dna_domain_fastq::parse_effective_params;
use bijux_dna_stage_contract::StagePlanV1;

mod fastqc;
mod filters;

use fastqc::fastqc_metrics_v2_from_dir;
use filters::{
    filter_metrics_with_removals, filter_removals_from_bbduk_stats, filter_removals_from_fastp,
    parse_screen_report, FilterRemovalCounts,
};

include!("sections/stage_metrics.rs");
include!("sections/envelope_and_stats.rs");
