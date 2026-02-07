//! FASTQ stage wiring (internal).

mod correct;
mod filter;
mod merge;
mod preprocess;
mod qc_post;
mod screen;
mod trim;
mod umi;
mod validate_pre;

pub use correct::*;
pub use filter::*;
pub use merge::*;
pub use preprocess::*;
pub use qc_post::*;
pub use screen::*;
pub use trim::*;
pub use umi::*;
pub use validate_pre::*;
