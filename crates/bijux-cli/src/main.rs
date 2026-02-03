// split to keep module size manageable

mod adapter_bank;
#[path = "main/bam_plan.rs"]
mod bam_plan;
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

pub(crate) use bam_plan::{plan_for_bam_stage, plan_for_bam_stage_with_profile};

include!("main/entry.rs");
include!("main/fastq.rs");
include!("main/bench.rs");
include!("main/other.rs");
include!("main/bam.rs");
