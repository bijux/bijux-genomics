pub mod align;
pub mod core;
pub mod downstream;
pub mod pre;

pub use align::{bowtie2, bwa};
pub use core::{mapdamage2, mosdepth, preseq, pydamage};
pub use downstream::{authenticct, gatk, rxy};
pub use pre::samtools;

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
