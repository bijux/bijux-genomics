//! Pre-alignment tool adapter implementations for BAM pipelines.

pub mod bamtools;
pub mod bedtools;
pub mod bowtie2;
pub mod bwa;
pub mod samtools;

#[must_use]
pub const fn module_name() -> &'static str {
    "pre"
}
