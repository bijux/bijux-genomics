//! Owner: bijux-dna-bench-workflows
//! Shared output ownership and path/reference preparation.
pub(crate) mod output;
pub(crate) mod paths;
pub(crate) mod stage;
#[cfg(feature = "genomics")]
pub(crate) mod variants;
