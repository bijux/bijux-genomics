use bijux_core::prelude::params_hash;

// Owner: bijux-analyze
// `SQLite` query helpers for benchmark storage.

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::aggregate::{
    BenchmarkRecord, FastqFilterMetrics, FastqTrimMetrics, FastqValidateMetrics,
};
use crate::aggregate::metrics::{
    ImageQaRecord, IMAGE_QA_INPUTS_SCHEMA_VERSION, IMAGE_QA_SCHEMA_VERSION,
};
use crate::{
    FastqCorrectMetrics, FastqMergeMetrics, FastqQcPostMetrics, FastqScreenMetrics,
    FastqStatsMetrics, FastqUmiMetrics,
};

use super::rows::benchmark_record_from_row;
use super::{
    ensure_identity_index, ensure_image_qa_identity_index, ensure_inserted_at_column,
    ensure_params_hash_column, ensure_record_id_column,
};

include!("queries_core_trim.rs");
include!("queries_core_other.rs");
