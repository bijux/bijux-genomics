#![allow(clippy::large_enum_variant, clippy::struct_excessive_bools)]

// split to keep module size manageable

include!("parse/parse_root_and_analyze.rs");
include!("parse/parse_env_and_pipeline.rs");
include!("parse/fixtures.rs");
mod bench;
include!("parse/ci.rs");
include!("parse/fastq.rs");
include!("parse/bam.rs");
include!("parse/vcf.rs");

pub use self::bench::*;
