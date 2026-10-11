//! Owner: bijux-dna-bench-workflows
//! Core germline execution responsibilities.
mod artifacts;
pub(crate) use artifacts::read_json_document;
mod timing;
pub(crate) use timing::timestamp_marker;
pub(crate) mod stages;
