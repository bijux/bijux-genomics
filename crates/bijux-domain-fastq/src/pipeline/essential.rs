#[must_use]
pub fn essential_stages() -> Vec<&'static str> {
    vec![
        "fastq.validate",
        "fastq.trim",
        "fastq.filter",
        "fastq.stats",
    ]
}
