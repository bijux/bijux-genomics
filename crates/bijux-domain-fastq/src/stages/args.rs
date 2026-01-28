use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BenchFastqTrimArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqValidateArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub strict: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqFilterArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqMergeArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqCorrectArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqQc2Args {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqUmiArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqScreenArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqStatsArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
}

#[derive(Debug, Clone)]
pub struct BenchFastqPreprocessArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub strict: bool,
}
