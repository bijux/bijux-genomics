//! Owner: bijux-dna-bench-workflows
//! Core germline observations responsibilities.
mod truth;
pub(crate) use truth::{build_profile_report, write_filter_reads_report};
mod variants;
pub(crate) use variants::required_tool_id;
mod reads;
pub(crate) use reads::{
    combined_gc_percent, combined_mean_quality, length_histogram, read_fastq_records, total_bases,
    write_fastq_records,
};
pub(crate) use truth::FilterReadPaths;
