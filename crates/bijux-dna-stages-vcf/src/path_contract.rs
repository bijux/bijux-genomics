use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct VcfPathContract {
    pub vcf_gz: PathBuf,
    pub vcf_gz_tbi: PathBuf,
    pub manifest: PathBuf,
    pub logs_dir: PathBuf,
}

impl VcfPathContract {
    #[must_use]
    pub fn for_stage(stage_dir: &Path, base_name: &str) -> Self {
        let vcf_gz = stage_dir.join(format!("{base_name}.vcf.gz"));
        let vcf_gz_tbi = PathBuf::from(format!("{}.tbi", vcf_gz.display()));
        Self {
            vcf_gz,
            vcf_gz_tbi,
            manifest: stage_dir.join("stage_manifest.json"),
            logs_dir: stage_dir.join("logs"),
        }
    }

    #[must_use]
    pub fn chunk_logs_dir(&self, stage_id: &str, chunk_label: &str) -> PathBuf {
        self.logs_dir.join(stage_id).join(chunk_label)
    }
}
