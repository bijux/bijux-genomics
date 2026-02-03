#[derive(Debug, Clone)]
pub struct CanonicalPipeline {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

#[must_use]
pub fn canonical_pipeline() -> CanonicalPipeline {
    CanonicalPipeline {
        required: vec![
            "fastq.validate_pre".to_string(),
            "fastq.detect_adapters".to_string(),
            "fastq.trim".to_string(),
            "fastq.filter".to_string(),
            "fastq.stats_neutral".to_string(),
            "fastq.qc_post".to_string(),
        ],
        optional: vec![
            "fastq.merge".to_string(),
            "fastq.correct".to_string(),
            "fastq.umi".to_string(),
            "fastq.screen".to_string(),
        ],
    }
}
