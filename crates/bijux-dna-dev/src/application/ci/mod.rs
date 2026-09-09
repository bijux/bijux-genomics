//! Admission of declared test inventories and observed CI execution evidence.

mod coverage;
mod events;
mod inventory;
#[cfg(test)]
mod tests;

pub(crate) use coverage::CoverageAttestation;
pub(crate) use inventory::ExpectedTestInventory;
