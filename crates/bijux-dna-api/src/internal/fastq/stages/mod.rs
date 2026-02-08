//! FASTQ stage wiring (internal).

pub(crate) mod correct;
#[path = "../filter.rs"]
pub(crate) mod filter;
#[path = "../merge.rs"]
pub(crate) mod merge;
#[path = "../preprocess.rs"]
pub(crate) mod preprocess;
#[path = "../qc_post.rs"]
pub(crate) mod qc_post;
#[path = "../screen.rs"]
pub(crate) mod screen;
#[path = "../trim.rs"]
pub(crate) mod trim;
#[path = "../umi.rs"]
pub(crate) mod umi;
#[path = "../validate_pre.rs"]
pub(crate) mod validate_pre;
