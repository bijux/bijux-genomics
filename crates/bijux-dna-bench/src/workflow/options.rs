//! Owner: bijux-dna-bench
//! Durable configuration for benchmark workflow execution.

#[derive(Debug, Clone)]
pub struct BenchRunOptions {
    pub output_dir: Option<std::path::PathBuf>,
    pub resume: bool,
    pub force: bool,
    pub ci_bootstrap: Option<usize>,
    pub objective: String,
}

impl Default for BenchRunOptions {
    fn default() -> Self {
        Self {
            output_dir: None,
            resume: false,
            force: false,
            ci_bootstrap: None,
            objective: "balanced".to_string(),
        }
    }
}
