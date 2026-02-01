use bijux_core::domain::PipelineSpec;

use super::canonical::canonical_pipeline;

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct DefaultPipelineOptions {
    pub paired: bool,
    pub enable_merge: bool,
    pub enable_correct: bool,
    pub enable_qc_post: bool,
    pub enable_screen: bool,
}

impl Default for DefaultPipelineOptions {
    fn default() -> Self {
        Self {
            paired: false,
            enable_merge: true,
            enable_correct: true,
            enable_qc_post: true,
            enable_screen: false,
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
    if options.enable_screen && !stages.iter().any(|stage| stage == "fastq.screen") {
        stages.push("fastq.screen".to_string());
    }
    if options.enable_qc_post && !stages.iter().any(|stage| stage == "fastq.qc_post") {
        stages.push("fastq.qc_post".to_string());
    }
    PipelineSpec { stages }
}
