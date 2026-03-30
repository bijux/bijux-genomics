//! Owner: bijux-dna-domain-fastq
//! Canonical effective parameters for FASTQ stages.

pub mod defaults;
mod descriptor;
pub mod edna;
mod effective;
mod parsing;
pub mod processing;
pub mod quality;

pub use descriptor::{stage_param_descriptor, StageParamDescriptor};
pub use effective::{DamageMode, EffectiveParams, PairedMode};
pub use parsing::parse_effective_params;
pub use processing::{correct, merge, preprocess, reference_index, remove_duplicates, umi};
pub use quality::{detect_adapters, filter, qc_post, screen, stats, trim, validate};
