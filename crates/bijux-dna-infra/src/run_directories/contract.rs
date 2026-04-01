use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RunLayoutPaths {
    pub run_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct RunLayoutContract {
    pub runs_dir: &'static str,
    pub artifacts_dir: &'static str,
    pub logs_dir: &'static str,
    pub tmp_dir: &'static str,
    pub lock_file: &'static str,
    pub publish_marker: &'static str,
}

pub const RUN_LAYOUT_CONTRACT: RunLayoutContract = RunLayoutContract {
    runs_dir: "runs",
    artifacts_dir: "artifacts",
    logs_dir: "logs",
    tmp_dir: "tmp",
    lock_file: ".run.lock",
    publish_marker: "published.json",
};

pub const PIPELINE_RUN_DIR_TEMPLATE: &str = "{pipeline_id}/{sample_id}/{run_id}";
