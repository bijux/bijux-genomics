use bijux_engine::api::PipelineSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCriticality {
    Essential,
    Optional,
    Experimental,
}

#[must_use]
pub fn canonical_stage_order() -> Vec<&'static str> {
    vec![
        "fastq.validate_pre",
        "fastq.trim",
        "fastq.filter",
        "fastq.stats_neutral",
    ]
}

#[must_use]
pub fn optional_branches() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        ("fastq.merge", &["fastq.trim", "fastq.filter"]),
        ("fastq.correct", &["fastq.trim"]),
        ("fastq.umi", &["fastq.trim"]),
        ("fastq.qc_post", &["fastq.validate_pre"]),
        ("fastq.screen", &["fastq.validate_pre"]),
    ]
}

#[must_use]
pub fn forbidden_transitions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("fastq.validate_pre", "fastq.merge"),
        ("fastq.stats_neutral", "fastq.trim"),
        ("fastq.stats_neutral", "fastq.filter"),
        ("fastq.stats_neutral", "fastq.merge"),
    ]
}

#[must_use]
pub fn stage_criticality(stage_id: &str) -> Option<StageCriticality> {
    match stage_id {
        "fastq.validate_pre"
        | "fastq.trim"
        | "fastq.merge"
        | "fastq.correct"
        | "fastq.filter"
        | "fastq.stats_neutral" => Some(StageCriticality::Essential),
        "fastq.qc_post" | "fastq.umi" | "fastq.preprocess" => Some(StageCriticality::Optional),
        "fastq.screen" => Some(StageCriticality::Experimental),
        _ => None,
    }
}

#[must_use]
pub fn preprocess_pipeline() -> PipelineSpec {
    PipelineSpec {
        stages: canonical_stage_order()
            .into_iter()
            .map(str::to_string)
            .collect(),
    }
}
