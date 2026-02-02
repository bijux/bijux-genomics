pub mod authenticct;
pub mod mosdepth;
pub mod preseq;
pub mod pydamage;
pub mod rxy;
pub mod samtools;

#[must_use]
pub fn available_tools() -> &'static [&'static str] {
    &[
        "samtools",
        "mosdepth",
        "preseq",
        "pydamage",
        "authenticct",
        "rxy",
    ]
}
