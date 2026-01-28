use bijux_engine::api::PipelineSpec;

#[derive(Debug, Clone, Copy)]
pub struct DefaultPipelineOptions {
    pub paired: bool,
    pub enable_merge: bool,
    pub enable_correct: bool,
}

impl Default for DefaultPipelineOptions {
    fn default() -> Self {
        Self {
            paired: false,
            enable_merge: true,
            enable_correct: true,
        }
    }
}

#[must_use]
pub fn fastq_default_pipeline(options: DefaultPipelineOptions) -> PipelineSpec {
    let mut stages = vec!["fastq.validate_pre".to_string(), "fastq.trim".to_string()];
    if options.paired && options.enable_correct {
        stages.push("fastq.correct".to_string());
    }
    if options.paired && options.enable_merge {
        stages.push("fastq.merge".to_string());
    }
    stages.push("fastq.filter".to_string());
    stages.push("fastq.stats_neutral".to_string());
    PipelineSpec { stages }
}
