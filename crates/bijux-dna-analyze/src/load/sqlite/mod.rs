//! Owner: bijux-dna-analyze
//! `SQLite` connection helpers for benchmark storage.

mod connection;
mod queries;
mod rows;
mod schema;

pub use connection::open_sqlite;
pub use queries::*;

pub(super) use connection::json_from_str;
pub(super) use schema::{
    ensure_identity_index, ensure_image_qa_identity_index, ensure_inserted_at_column,
    ensure_params_hash_column, ensure_record_id_column,
};
