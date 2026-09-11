/// Nonserialized observations preserve downstream count and missingness evidence.
#[derive(Debug, Clone)]
pub struct VariantCounts {
    pub stage_id: String,
    pub tool_id: String,
    pub coverage_status: String,
    pub variant_count: u64,
    pub sample_count: u64,
}
#[derive(Debug, Clone)]
pub struct VariantMissingness {
    pub stage_id: String,
    pub tool_id: String,
    pub coverage_status: String,
    pub sample_missingness_count: usize,
    pub variant_missingness_count: usize,
    pub sample_exclusion_threshold: f64,
    pub variant_exclusion_threshold: f64,
}
#[derive(Debug, Clone)]
pub struct PublishedVariantCounts {
    pub output_path: String,
    pub rows: Vec<VariantCounts>,
}
#[derive(Debug, Clone)]
pub struct PublishedVariantMissingness {
    pub output_path: String,
    pub rows: Vec<VariantMissingness>,
}
