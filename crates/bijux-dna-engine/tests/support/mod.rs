//! Shared test helpers for bijux-dna-engine.

#![allow(dead_code)]

mod execution_setup;
mod manifest_fixture;
mod plan_factory;
mod runner_stub;

#[allow(unused_imports)]
pub use execution_setup::execution_setup;
#[allow(unused_imports)]
pub use manifest_fixture::{layout_tree_text, write_manifest_hash};
#[allow(unused_imports)]
pub use plan_factory::{build_graph, plan_for};
#[allow(unused_imports)]
pub use runner_stub::{DeterministicRunner, FakeRunner, RecordingRunner};
