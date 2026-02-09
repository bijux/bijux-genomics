//! Pre-alignment tool adapter implementations for BAM pipelines.

pub mod bamtools;
pub mod bedtools;
pub mod bowtie2;
pub mod bwa;
pub mod samtools;

pub const PRE_TOOL_IDS: &[&str] = &["bamtools", "bedtools", "bowtie2", "bwa", "samtools"];
