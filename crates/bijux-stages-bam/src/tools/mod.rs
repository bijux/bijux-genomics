pub mod align;
pub mod authenticct;
pub mod gatk;
pub mod mapdamage2;
pub mod mosdepth;
pub mod preseq;
pub mod pydamage;
pub mod rxy;
pub mod samtools;

pub use align::bowtie2;
pub use align::bwa;

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
