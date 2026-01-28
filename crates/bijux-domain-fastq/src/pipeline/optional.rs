#[derive(Debug, Clone, Copy)]
pub struct OptionalStage {
    pub stage_id: &'static str,
    pub prerequisites: &'static [&'static str],
    pub mutates_fastq: bool,
    pub affects_metrics: bool,
}

#[must_use]
pub fn optional_stages() -> Vec<OptionalStage> {
    vec![
        OptionalStage {
            stage_id: "fastq.merge",
            prerequisites: &["fastq.trim", "fastq.filter"],
            mutates_fastq: true,
            affects_metrics: true,
        },
        OptionalStage {
            stage_id: "fastq.correct",
            prerequisites: &["fastq.trim"],
            mutates_fastq: true,
            affects_metrics: true,
        },
        OptionalStage {
            stage_id: "fastq.umi",
            prerequisites: &["fastq.trim"],
            mutates_fastq: true,
            affects_metrics: true,
        },
        OptionalStage {
            stage_id: "fastq.screen",
            prerequisites: &["fastq.validate"],
            mutates_fastq: false,
            affects_metrics: false,
        },
        OptionalStage {
            stage_id: "fastq.qc_post",
            prerequisites: &["fastq.validate"],
            mutates_fastq: false,
            affects_metrics: false,
        },
    ]
}
