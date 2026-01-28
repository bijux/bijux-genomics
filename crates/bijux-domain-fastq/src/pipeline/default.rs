use bijux_core::domain::PipelineSpec;

use super::canonical::canonical_pipeline;

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
    let canonical = canonical_pipeline();
    let mut stages = canonical.required;
    if options.paired && options.enable_correct {
        stages.push("fastq.correct".to_string());
    }
    if options.paired && options.enable_merge {
        stages.push("fastq.merge".to_string());
    }
    PipelineSpec { stages }
}
