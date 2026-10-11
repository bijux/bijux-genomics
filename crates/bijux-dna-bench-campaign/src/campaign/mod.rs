//! Corpus campaign lifecycle over admitted input records and explicit execution ports.
mod admission;
pub(crate) mod paths;
mod resume;
mod service;
pub use service::run_corpus_campaign;

#[cfg(test)]
mod contracts;

mod execution;
mod model;
mod publication;
mod samples;
