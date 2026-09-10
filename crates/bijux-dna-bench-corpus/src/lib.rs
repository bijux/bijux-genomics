//! Owner: bijux-dna-bench-corpus
//! Local benchmark corpus admission, independent of commands and campaign orchestration.
pub mod amplicon;
#[cfg(feature = "alignment")]
pub mod bam;
pub mod damage;
pub mod edna;
pub mod fastq;
mod paths;
#[cfg(feature = "variants")]
pub mod vcf;
