//! Owner: bijux-dna-runner
//! Non-waiting Slurm backend.

mod benchmark;
mod client;
mod contracts;
mod corpus_materialization;
mod parsing;
mod qualification;
mod stable_surface;
mod submission;

pub use stable_surface::*;
