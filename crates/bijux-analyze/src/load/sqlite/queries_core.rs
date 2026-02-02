// Owner: bijux-analyze
// `SQLite` query helpers for benchmark storage.

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::aggregate::{
    BenchmarkRecord, FastqFilterMetrics, FastqTrimMetrics, FastqValidateMetrics,
};

use super::rows::benchmark_record_from_row;
use super::{
    ensure_identity_index, ensure_inserted_at_column, ensure_params_hash_column,
    ensure_record_id_column, params_hash,
};

include!("queries_core_trim.rs");
include!("queries_core_other.rs");
