pub mod catalog;
pub mod core;
pub mod downstream;
pub mod pre;

pub use catalog::TOOLS_NAMESPACE;
pub use core::{mapdamage2, mosdepth, preseq, pydamage};
pub use downstream::{authenticity, gatk, rxy};
pub use pre::{bowtie2, bwa, samtools};

#[must_use]
pub fn available_tools() -> &'static [&'static str] {
    &[
        "samtools",
        "mosdepth",
        "preseq",
        "pydamage",
        "mapdamage2",
        "bwa",
        "bowtie2",
        "gatk",
        "authenticct",
        "rxy",
    ]
}
