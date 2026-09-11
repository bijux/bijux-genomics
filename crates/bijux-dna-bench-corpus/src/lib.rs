//! Owner: bijux-dna-bench-corpus
//! Local benchmark corpus admission, independent of commands and campaign orchestration.
pub mod amplicon;
#[cfg(feature = "alignment")]
pub mod bam;
pub mod damage;
pub mod edna;
pub mod fastq;
#[cfg(feature = "panel-metadata")]
pub mod panel_metadata;
mod paths;
#[cfg(feature = "variants")]
pub mod vcf;

#[cfg(feature = "source-contract")]
pub mod source;

#[cfg(feature = "normalized")]
pub mod normalized;
