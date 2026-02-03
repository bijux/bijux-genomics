// split to keep module size manageable

mod cli;
mod env;
mod main_helpers;

include!("main/entry.rs");
include!("main/fastq.rs");
include!("main/bench.rs");
include!("main/other.rs");
include!("main/bam.rs");
