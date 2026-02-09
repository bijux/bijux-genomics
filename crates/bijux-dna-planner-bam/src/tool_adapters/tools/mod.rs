pub mod catalog;
pub mod core;
pub mod downstream;
pub mod pre;

pub use catalog::TOOLS_NAMESPACE;
pub use core::{
    addeam, damageprofiler, mapdamage2, mosdepth, ngsbriggs, pmdtools, preseq, pydamage,
};
pub use downstream::{authenticity, contammix, gatk, rxy, schmutzi, verifybamid2};
pub use pre::{bamtools, bedtools, bowtie2, bwa, samtools};

#[must_use]
pub fn available_tools() -> &'static [&'static str] {
    &[
        "addeam",
        "bamtools",
        "bedtools",
        "samtools",
        "mosdepth",
        "preseq",
        "pydamage",
        "pmdtools",
        "damageprofiler",
        "ngsbriggs",
        "mapdamage2",
        "bwa",
        "bowtie2",
        "gatk",
        "authenticct",
        "schmutzi",
        "verifybamid2",
        "contammix",
        "rxy",
    ]
}
