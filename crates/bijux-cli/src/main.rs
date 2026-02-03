// split to keep module size manageable

mod adapter_bank;
mod bam_router;
mod cli;
mod contaminant_bank;
mod cross_router;
mod env;
mod fastq_exec;
mod fastq_router;
mod fastq_stats_neutral;
mod main_helpers;
mod polyx_bank;
mod replay;
mod utils;

include!("main/entry.rs");
include!("main/fastq.rs");
include!("main/bench.rs");
include!("main/other.rs");
include!("main/bam.rs");
