//! Pure benchmark taxonomy summary admission and expected-result evaluation.
mod observations;
pub use observations::decode_benchmark_taxonomy_observation;
mod evaluation;
pub use evaluation::evaluate_benchmark_taxonomy_outputs;

#[cfg(test)]
#[path = "../../../tests/contracts/taxonomy_judgment/mod.rs"]
mod tests;
